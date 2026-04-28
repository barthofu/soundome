use domain::ports::repositories::ArtistRepository;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::{models::{Artist, Reference}, types::SoundomeResult};

use crate::{
    delete_with_relations, entities::{
        ArtistAlbumEntity, ArtistEntity, ArtistRefEntity, ArtistTrackEntity, NewArtistEntity, NewArtistRefEntity, UpdateArtistEntity
    }, schema
};

use crate::diesel::Connection;

pub struct DieselArtistRepository {}

impl DieselArtistRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl ArtistRepository for DieselArtistRepository {

    // =================================================================================
    // Custom
    // =================================================================================

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Artist> {
        let artist_ref = schema::artist_ref::table
            .filter(schema::artist_ref::external_url.eq(url))
            .first::<ArtistRefEntity>(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by url: {}",
                    err
                ))
            })?;

        self.get_by_id(conn, artist_ref.artist_id)
    }

    fn create_references(&self, conn: &mut SqliteConnection, artist_id: i32, references: &[Reference]) -> SoundomeResult<()> {
        for reference in references {
            let new_artist_ref = NewArtistRefEntity::convert_from_domain(reference, artist_id);
            
            diesel::insert_into(schema::artist_ref::table)
                .values(&new_artist_ref)
                .execute(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to create artist reference: {}",
                        err
                    ))
                })?;
        }
        Ok(())
    }

    fn set_references(&self, conn: &mut SqliteConnection, artist_id: i32, references: &[Reference]) -> SoundomeResult<()> {
        // Merge semantics: keep existing rows (and their ids), only insert missing refs.
        if references.is_empty() {
            return Ok(());
        }

        let existing: Vec<ArtistRefEntity> = schema::artist_ref::table
            .filter(schema::artist_ref::artist_id.eq(artist_id))
            .load(conn)
            .map_err(|err| shared::errors::Error::Database(format!("Failed to load artist references: {}", err)))?;

        for reference in references {
            if reference.external_id.is_none() && reference.external_url.is_none() {
                continue;
            }

            let ref_type = reference.ref_type.as_ref().to_string().to_lowercase();
            let platform = reference.platform.as_ref().to_string().to_lowercase();

            let already_exists = existing.iter().any(|r| {
                r.ref_type.to_lowercase() == ref_type
                    && r.platform.to_lowercase() == platform
                    && r.external_id == reference.external_id
                    && r.external_url == reference.external_url
            });

            if !already_exists {
                let new_artist_ref = NewArtistRefEntity::convert_from_domain(reference, artist_id);
                diesel::insert_into(schema::artist_ref::table)
                    .values(&new_artist_ref)
                    .execute(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to create artist reference: {}",
                            err
                        ))
                    })?;
            }
        }

        Ok(())
    }

    fn create_track_relationship(&self, conn: &mut SqliteConnection, artist_id: i32, track_id: i32) -> SoundomeResult<()> {
        let artist_track = ArtistTrackEntity {
            track_id,
            artist_id,
        };
        
        diesel::insert_into(schema::artist_tracks::table)
            .values(&artist_track)
            .execute(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to create artist-track relationship: {}",
                    err
                ))
            })?;
        Ok(())
    }

    fn set_track_artists(&self, conn: &mut SqliteConnection, track_id: i32, artist_ids: &[i32]) -> SoundomeResult<()> {
        // clear existing
        diesel::delete(schema::artist_tracks::table.filter(schema::artist_tracks::track_id.eq(track_id)))
            .execute(conn)
            .map_err(|err| shared::errors::Error::Database(format!("Failed to clear artist-track relationships: {}", err)))?;
        // insert current
        for artist_id in artist_ids {
            let rel = ArtistTrackEntity { track_id, artist_id: *artist_id };
            diesel::insert_into(schema::artist_tracks::table)
                .values(&rel)
                .execute(conn)
                .map_err(|err| shared::errors::Error::Database(format!("Failed to create artist-track relationship: {}", err)))?;
        }
        Ok(())
    }

    fn create_album_relationship(&self, conn: &mut SqliteConnection, artist_id: i32, album_id: i32) -> SoundomeResult<()> {
        let artist_album = ArtistAlbumEntity {
            album_id,
            artist_id,
        };
        
        diesel::insert_into(schema::artist_albums::table)
            .values(&artist_album)
            .execute(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to create artist-album relationship: {}",
                    err
                ))
            })?;
        Ok(())
    }

    fn set_album_artists(&self, conn: &mut SqliteConnection, album_id: i32, artist_ids: &[i32]) -> SoundomeResult<()> {
        // clear existing
        diesel::delete(schema::artist_albums::table.filter(schema::artist_albums::album_id.eq(album_id)))
            .execute(conn)
            .map_err(|err| shared::errors::Error::Database(format!("Failed to clear artist-album relationships: {}", err)))?;
        // insert current
        for artist_id in artist_ids {
            let rel = ArtistAlbumEntity { album_id, artist_id: *artist_id };
            diesel::insert_into(schema::artist_albums::table)
                .values(&rel)
                .execute(conn)
                .map_err(|err| shared::errors::Error::Database(format!("Failed to create artist-album relationship: {}", err)))?;
        }
        Ok(())
    }

    fn create_or_ignore(&self, conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Artist> {
        // If artist already has an ID, return it as-is
        if let Some(id) = artist.id {
            return self.get_by_id(conn, id);
        }
        // Exact-name fast path
        let exact: Option<ArtistEntity> = schema::artist::table
            .filter(schema::artist::name.eq(&artist.name))
            .first(conn)
            .optional()
            .map_err(|err| shared::errors::Error::Database(format!("Failed to look up artist: {}", err)))?;
        if let Some(entity) = exact {
            let references: Vec<ArtistRefEntity> = schema::artist_ref::table
                .filter(schema::artist_ref::artist_id.eq(entity.id))
                .load(conn)
                .unwrap_or_default();
            return Ok(ArtistEntity::convert_to_domain(entity, references));
        }
        // Case-insensitive fallback (Unicode-safe: compare lowercased in Rust)
        let name_lower = artist.name.to_lowercase();
        let all: Vec<ArtistEntity> = schema::artist::table
            .load(conn)
            .map_err(|err| shared::errors::Error::Database(format!("Failed to load artists for dedup: {}", err)))?;
        if let Some(entity) = all.into_iter().find(|e| e.name.to_lowercase() == name_lower) {
            let references: Vec<ArtistRefEntity> = schema::artist_ref::table
                .filter(schema::artist_ref::artist_id.eq(entity.id))
                .load(conn)
                .unwrap_or_default();
            return Ok(ArtistEntity::convert_to_domain(entity, references));
        }
        // Not found: create the artist and its references
        let created_artist = self.create(conn, artist)?;
        let artist_id = created_artist.id.unwrap();
        self.create_references(conn, artist_id, &artist.references)?;
        self.get_by_id(conn, artist_id)
    }

    // fn find_by_unique_fields(&self, conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Option<Artist>> {
    //     use diesel::prelude::*;
    //     use crate::schema;
    //     use crate::schema::artist::dsl::*;
    //     let found: Option<ArtistEntity> = artist
    //         .filter(name.eq(&artist.name))
    //         .first::<ArtistEntity>(conn)
    //         .optional()
    //         .map_err(|err| shared::errors::Error::Database(format!("Failed to find artist by unique fields: {}", err)))?;
    //     if let Some(entity) = found {
    //         let references: Vec<ArtistRefEntity> = schema::artist_ref::table
    //             .filter(schema::artist_ref::artist_id.eq(entity.id))
    //             .load(conn)
    //             .unwrap_or_default();
    //         Ok(Some(ArtistEntity::convert_to_domain(entity, references)))
    //     } else {
    //         Ok(None)
    //     }
    // }

    // =================================================================================
    // CRUD
    // =================================================================================

    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Artist> {
        let artist: ArtistEntity = schema::artist::table
            .filter(schema::artist::id.eq(id))
            .first(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let references: Vec<ArtistRefEntity> = schema::artist_ref::table
            .filter(schema::artist_ref::artist_id.eq(artist.id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        Ok(ArtistEntity::convert_to_domain(
            artist,
            references,
        ))
    }

    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Artist>> {
        let artists: Vec<ArtistEntity> = schema::artist::table
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get all resources: {}",
                    err
                ))
            })?;

        let references: Vec<ArtistRefEntity> = schema::artist_ref::table
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get all resources: {}",
                    err
                ))
            })?;

        Ok(artists.into_iter()
            .map(|artist| ArtistEntity::convert_to_domain(
                artist,
                references.clone(),
            ))
            .collect())
    }

    fn create(&self, conn: &mut SqliteConnection, new_artist: &Artist) -> SoundomeResult<Artist> {
        let new_artist_entity = NewArtistEntity::convert_from_domain(new_artist);
        let inserted_artist = diesel::insert_into(schema::artist::table)
            .values(&new_artist_entity)
            .execute(conn)
            .and_then(|_| {
                schema::artist::table
                    .order(schema::artist::id.desc())
                    .first::<ArtistEntity>(conn)
            })
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to create resource: {}",
                    err
                ))
            })?;

        Ok(ArtistEntity::convert_to_domain(
            inserted_artist,
            vec![],
        ))
    }

    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_artist: &Artist) -> SoundomeResult<Artist> {
        let updated_artist_entity = UpdateArtistEntity::convert_from_domain(updated_artist);
        let updated_artist = diesel::update(schema::artist::table.filter(schema::artist::id.eq(id)))
            .set(&updated_artist_entity)
            .execute(conn)
            .and_then(|_| {
                schema::artist::table
                    .filter(schema::artist::id.eq(id))
                    .first::<ArtistEntity>(conn)
            })
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to update resource: {}",
                    err
                ))
            })?;

        Ok(ArtistEntity::convert_to_domain(
            updated_artist,
            vec![],
        ))
    }

    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
                
        delete_with_relations!(
            conn,
            id,
            [
                (schema::artist_ref::table, schema::artist_ref::artist_id, "Failed to delete associated artist references"),
                (schema::artist_albums::table, schema::artist_albums::artist_id, "Failed to delete associated artist-album relationships"),
                (schema::artist_tracks::table, schema::artist_tracks::artist_id, "Failed to delete associated artist-track relationships"),
                (schema::artist::table, schema::artist::id, "Failed to delete resource"),
            ]
        )?;
        Ok(())
    }

    fn merge_into(&self, conn: &mut SqliteConnection, source_ids: &[i32], target_id: i32) -> SoundomeResult<()> {
        use diesel::Connection as _;
        conn.transaction(|conn| {
            // --- Re-point artist_tracks -------------------------------------------------
            // Load all track IDs already linked to the target to avoid duplicate PK insertion.
            let target_track_ids: Vec<i32> = schema::artist_tracks::table
                .filter(schema::artist_tracks::artist_id.eq(target_id))
                .select(schema::artist_tracks::track_id)
                .load(conn)
                .map_err(|e| shared::errors::Error::Database(format!("merge: load target tracks: {e}")))?;

            for &src in source_ids {
                // Fetch track ids linked to this source artist.
                let src_track_ids: Vec<i32> = schema::artist_tracks::table
                    .filter(schema::artist_tracks::artist_id.eq(src))
                    .select(schema::artist_tracks::track_id)
                    .load(conn)
                    .map_err(|e| shared::errors::Error::Database(format!("merge: load source tracks: {e}")))?;

                for track_id in src_track_ids {
                    if !target_track_ids.contains(&track_id) {
                        diesel::insert_into(schema::artist_tracks::table)
                            .values(ArtistTrackEntity { track_id, artist_id: target_id })
                            .execute(conn)
                            .map_err(|e| shared::errors::Error::Database(format!("merge: insert artist_track: {e}")))?;
                    }
                }
                diesel::delete(schema::artist_tracks::table.filter(schema::artist_tracks::artist_id.eq(src)))
                    .execute(conn)
                    .map_err(|e| shared::errors::Error::Database(format!("merge: delete source artist_tracks: {e}")))?;
            }

            // --- Re-point artist_albums -------------------------------------------------
            let target_album_ids: Vec<i32> = schema::artist_albums::table
                .filter(schema::artist_albums::artist_id.eq(target_id))
                .select(schema::artist_albums::album_id)
                .load(conn)
                .map_err(|e| shared::errors::Error::Database(format!("merge: load target albums: {e}")))?;

            for &src in source_ids {
                let src_album_ids: Vec<i32> = schema::artist_albums::table
                    .filter(schema::artist_albums::artist_id.eq(src))
                    .select(schema::artist_albums::album_id)
                    .load(conn)
                    .map_err(|e| shared::errors::Error::Database(format!("merge: load source albums: {e}")))?;

                for album_id in src_album_ids {
                    if !target_album_ids.contains(&album_id) {
                        diesel::insert_into(schema::artist_albums::table)
                            .values(ArtistAlbumEntity { album_id, artist_id: target_id })
                            .execute(conn)
                            .map_err(|e| shared::errors::Error::Database(format!("merge: insert artist_album: {e}")))?;
                    }
                }
                diesel::delete(schema::artist_albums::table.filter(schema::artist_albums::artist_id.eq(src)))
                    .execute(conn)
                    .map_err(|e| shared::errors::Error::Database(format!("merge: delete source artist_albums: {e}")))?;
            }

            // --- Move artist_refs -------------------------------------------------------
            for &src in source_ids {
                diesel::update(schema::artist_ref::table.filter(schema::artist_ref::artist_id.eq(src)))
                    .set(schema::artist_ref::artist_id.eq(target_id))
                    .execute(conn)
                    .map_err(|e| shared::errors::Error::Database(format!("merge: move artist_ref: {e}")))?;
            }

            // --- Delete source artists --------------------------------------------------
            for &src in source_ids {
                diesel::delete(schema::artist::table.filter(schema::artist::id.eq(src)))
                    .execute(conn)
                    .map_err(|e| shared::errors::Error::Database(format!("merge: delete source artist: {e}")))?;
            }

            Ok(())
        })
    }

}
