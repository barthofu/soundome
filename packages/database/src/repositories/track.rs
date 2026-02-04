use domain::ports::repositories::TrackRepository;

use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use shared::{models::{Reference, Track}, types::SoundomeResult};

use crate::{
    delete_with_relations, entities::{AlbumEntity, ArtistEntity, NewTrackEntity, NewTrackRefEntity, TrackEntity, TrackRefEntity, UpdateTrackEntity}, schema
};

pub struct DieselTrackRepository {}

impl DieselTrackRepository {
    pub fn new() -> Self {
        Self { }
    }
}

impl TrackRepository for DieselTrackRepository {


    // =================================================================================
    // Custom
    // =================================================================================

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Track> {
        let track_ref = schema::track_ref::table
            .filter(schema::track_ref::external_url.eq(url))
            .first::<TrackRefEntity>(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by url: {}",
                    err
                ))
            })?;

        self.get_by_id(conn, track_ref.track_id)
    }

    fn create_references(&self, conn: &mut SqliteConnection, track_id: i32, references: &[Reference]) -> SoundomeResult<()> {
        for reference in references {
            let new_track_ref = NewTrackRefEntity::convert_from_domain(reference, track_id);
            
            diesel::insert_into(schema::track_ref::table)
                .values(&new_track_ref)
                .execute(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to create track reference: {}",
                        err
                    ))
                })?;
        }
        Ok(())
    }

    fn set_references(&self, conn: &mut SqliteConnection, track_id: i32, references: &[Reference]) -> SoundomeResult<()> {
        // Semantics:
        // - Source/Provider: replace (ensure single row): delete existing of that type then insert.
        // - Metadata/Reference: merge (insert missing only), preserving existing ids.
        if references.is_empty() {
            return Ok(());
        }

        // Handle Source and Provider replacement first
        for reference in references {
            let ref_type = reference.ref_type.as_ref().to_string().to_lowercase();
            if ref_type == "source" || ref_type == "provider" {
                diesel::delete(
                    schema::track_ref::table
                        .filter(schema::track_ref::track_id.eq(track_id))
                        .filter(schema::track_ref::type_.eq(&ref_type)),
                )
                .execute(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to replace track {} reference: {}",
                        ref_type, err
                    ))
                })?;

                if reference.external_id.is_none() && reference.external_url.is_none() {
                    continue;
                }

                let new_track_ref = NewTrackRefEntity::convert_from_domain(reference, track_id);
                diesel::insert_into(schema::track_ref::table)
                    .values(&new_track_ref)
                    .execute(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to create track {} reference: {}",
                            ref_type, err
                        ))
                    })?;
            }
        }

        // Then merge everything else
        let existing: Vec<TrackRefEntity> = schema::track_ref::table
            .filter(schema::track_ref::track_id.eq(track_id))
            .load(conn)
            .map_err(|err| shared::errors::Error::Database(format!("Failed to load track references: {}", err)))?;

        for reference in references {
            let ref_type = reference.ref_type.as_ref().to_string().to_lowercase();
            if ref_type == "source" || ref_type == "provider" {
                continue;
            }
            if reference.external_id.is_none() && reference.external_url.is_none() {
                continue;
            }

            let platform = reference.platform.as_ref().to_string().to_lowercase();

            let already_exists = existing.iter().any(|r| {
                r.ref_type.to_lowercase() == ref_type
                    && r.platform.to_lowercase() == platform
                    && r.external_id == reference.external_id
                    && r.external_url == reference.external_url
            });

            if !already_exists {
                let new_track_ref = NewTrackRefEntity::convert_from_domain(reference, track_id);
                diesel::insert_into(schema::track_ref::table)
                    .values(&new_track_ref)
                    .execute(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to create track reference: {}",
                            err
                        ))
                    })?;
            }
        }

        Ok(())
    }

    // fn find_by_unique_fields(&self, conn: &mut SqliteConnection, track: &Track) -> SoundomeResult<Option<Track>> {
    //     use diesel::prelude::*;
    //     use crate::schema;
    //     use crate::schema::track::dsl::*;
    //     let mut query = track.into_boxed();
    //     query = query.filter(title.eq(&track.));
    //     if let Some(album) = &track.album {
    //         if let Some(album_id_val) = album.id {
    //             query = query.filter(album_id.eq(album_id_val));
    //         }
    //     }
    //     let found: Option<TrackEntity> = query
    //         .first::<TrackEntity>(conn)
    //         .optional()
    //         .map_err(|err| shared::errors::Error::Database(format!("Failed to find track by unique fields: {}", err)))?;
    //     if let Some(entity) = found {
    //         let album = super::album::find_one(conn, entity.album_id.unwrap_or_default()).ok();
    //         let artists: Vec<ArtistEntity> = schema::artist_tracks::table
    //             .inner_join(schema::artist::table.on(schema::artist_tracks::artist_id.eq(schema::artist::id)))
    //             .filter(schema::artist_tracks::track_id.eq(entity.id))
    //             .select(schema::artist::all_columns)
    //             .load(conn)
    //             .unwrap_or_default();
    //         let references: Vec<TrackRefEntity> = schema::track_ref::table
    //             .filter(schema::track_ref::track_id.eq(entity.id))
    //             .load(conn)
    //             .unwrap_or_default();
    //         Ok(Some(TrackEntity::convert_to_domain(entity, album, artists, references)))
    //     } else {
    //         Ok(None)
    //     }
    // }

    // =================================================================================
    // CRUD
    // =================================================================================

    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Track> {
        let (track, album): (TrackEntity, Option<AlbumEntity>) = schema::track::table
            .left_join(schema::album::table.on(schema::album::id.nullable().eq(schema::track::album_id)))
            .filter(schema::track::id.eq(id))
            .first(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let artists: Vec<ArtistEntity> = schema::artist_tracks::table
            .inner_join(schema::artist::table.on(schema::artist_tracks::artist_id.eq(schema::artist::id)))
            .filter(schema::artist_tracks::track_id.eq(track.id))
            .select(schema::artist::all_columns)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let references: Vec<TrackRefEntity> = schema::track_ref::table
            .filter(schema::track_ref::track_id.eq(track.id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;
        
        Ok(TrackEntity::convert_to_domain(
            track,
            album,
            artists,
            references,
        ))
    }

    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>> {
        let tracks: Vec<TrackEntity> = schema::track::table
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get all resources: {}",
                    err
                ))
            })?;

        let mut result = Vec::new();
        for track in tracks {
            let album = if let Some(album_id) = track.album_id {
                schema::album::table
                    .filter(schema::album::id.eq(album_id))
                    .first::<AlbumEntity>(conn)
                    .ok()
            } else {
                None
            }; 

            let artists: Vec<ArtistEntity> = schema::artist_tracks::table
                .inner_join(schema::artist::table.on(schema::artist_tracks::artist_id.eq(schema::artist::id)))
                .filter(schema::artist_tracks::track_id.eq(track.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get all resources: {}",
                        err
                    ))
                })?;

            let references: Vec<TrackRefEntity> = schema::track_ref::table
                .filter(schema::track_ref::track_id.eq(track.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get all resources: {}",
                        err
                    ))
                })?;

            result.push(TrackEntity::convert_to_domain(
                track,
                album,
                artists,
                references,
            ));
        }

        Ok(result)
    }

    fn create(&self, conn: &mut SqliteConnection, new_track: &Track) -> SoundomeResult<Track> {
        let new_track_entity = NewTrackEntity::convert_from_domain(new_track);
        let inserted_track = diesel::insert_into(schema::track::table)
            .values(&new_track_entity)
            .execute(conn)
            .and_then(|_| {
                schema::track::table
                    .order(schema::track::id.desc())
                    .first::<TrackEntity>(conn)
            })
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to create resource: {}",
                    err
                ))
            })?;

        Ok(TrackEntity::convert_to_domain(
            inserted_track,
            None,
            vec![],
            vec![],
        ))
    }

    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &Track) -> SoundomeResult<Track> {
        let updated_track_entity = UpdateTrackEntity::convert_from_domain(updated_track);
        let updated_track = diesel::update(schema::track::table.filter(schema::track::id.eq(id)))
            .set(&updated_track_entity)
            .execute(conn)
            .and_then(|_| {
                schema::track::table
                    .filter(schema::track::id.eq(id))
                    .first::<TrackEntity>(conn)
            })
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to update resource: {}",
                    err
                ))
            })?;

        Ok(TrackEntity::convert_to_domain(
            updated_track,
            None,
            vec![],
            vec![],
        ))
    }

    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {

        delete_with_relations!(
            conn,
            id,
            [
                (schema::track_ref::table, schema::track_ref::track_id, "Failed to delete associated track references"),
                (schema::artist_tracks::table, schema::artist_tracks::track_id, "Failed to delete associated artist-track relationships"),
                (schema::track::table, schema::track::id, "Failed to delete resource"),
            ]
        )?;
        Ok(())
    }
}

