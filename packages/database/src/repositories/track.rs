use domain::ports::repositories::{Page, SortDir, TrackQuery, TrackRepository, TrackSortBy};
use std::collections::{HashMap, HashSet};

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Integer, Nullable, Text};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use shared::{
    models::{Reference, Track},
    types::SoundomeResult,
};

use crate::{
    delete_with_relations,
    entities::{
        AlbumEntity, ArtistEntity, ArtistTrackEntity, NewTrackEntity, NewTrackRefEntity,
        TrackEntity, TrackRefEntity, UpdateTrackEntity,
    },
    schema,
};

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = Integer)]
    id: i32,
}

#[derive(Default)]
pub struct DieselTrackRepository {}

impl DieselTrackRepository {
    pub fn new() -> Self {
        Self {}
    }

    /// Hydrate a page of tracks (album + artists + references) given an
    /// already-ordered list of ids, preserving that order.
    fn hydrate_tracks_by_ids(
        &self,
        conn: &mut SqliteConnection,
        ids: &[i32],
    ) -> SoundomeResult<Vec<Track>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let entities: Vec<TrackEntity> = schema::track::table
            .filter(schema::track::id.eq_any(ids.to_vec()))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to load tracks page: {}", err))
            })?;
        let mut by_id: std::collections::HashMap<i32, TrackEntity> =
            entities.into_iter().map(|e| (e.id, e)).collect();

        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            let Some(track) = by_id.remove(id) else {
                continue;
            };

            let album = if let Some(album_id) = track.album_id {
                schema::album::table
                    .filter(schema::album::id.eq(album_id))
                    .first::<AlbumEntity>(conn)
                    .ok()
            } else {
                None
            };

            let artists: Vec<ArtistEntity> = schema::artist_tracks::table
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_tracks::track_id.eq(track.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to load artists for track page: {}",
                        err
                    ))
                })?;

            let references: Vec<TrackRefEntity> = schema::track_ref::table
                .filter(schema::track_ref::track_id.eq(track.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to load references for track page: {}",
                        err
                    ))
                })?;

            result.push(TrackEntity::convert_to_domain(
                track, album, artists, references,
            ));
        }

        Ok(result)
    }

    /// Order+page track ids by their (first, alphabetically) linked artist name.
    /// Raw SQL: many-to-many artist join + aggregate ordering isn't expressible
    /// cleanly with the Diesel query builder for SQLite, so this is isolated here.
    fn ordered_track_ids_by_artist(
        &self,
        conn: &mut SqliteConnection,
        needs_validation: Option<bool>,
        search: Option<&str>,
        dir: SortDir,
        limit: i64,
        offset: i64,
    ) -> SoundomeResult<Vec<i32>> {
        let sql = format!(
            "SELECT t.id AS id \
             FROM track t \
             LEFT JOIN ( \
                 SELECT at.track_id AS track_id, MIN(ar.name) AS artist_name \
                 FROM artist_tracks at JOIN artist ar ON at.artist_id = ar.id \
                 GROUP BY at.track_id \
             ) ta ON ta.track_id = t.id \
             WHERE (? IS NULL OR t.needs_validation = ?) \
               AND (? IS NULL OR t.title LIKE ? OR t.id IN ( \
                     SELECT at2.track_id FROM artist_tracks at2 \
                     JOIN artist ar2 ON at2.artist_id = ar2.id \
                     WHERE ar2.name LIKE ? \
                   )) \
             ORDER BY ta.artist_name {} \
             LIMIT ? OFFSET ?",
            dir.as_sql()
        );

        let pattern = search.map(|s| format!("%{}%", s));

        let rows: Vec<IdRow> = diesel::sql_query(sql)
            .bind::<Nullable<Bool>, _>(needs_validation)
            .bind::<Nullable<Bool>, _>(needs_validation)
            .bind::<Nullable<Text>, _>(pattern.clone())
            .bind::<Nullable<Text>, _>(pattern.clone())
            .bind::<Nullable<Text>, _>(pattern)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to sort tracks by artist: {}", err))
            })?;

        Ok(rows.into_iter().map(|r| r.id).collect())
    }
}

impl TrackRepository for DieselTrackRepository {
    // =================================================================================
    // Custom
    // =================================================================================

    fn get_recent(&self, conn: &mut SqliteConnection, limit: i64) -> SoundomeResult<Vec<Track>> {
        let tracks: Vec<TrackEntity> = schema::track::table
            .order(schema::track::id.desc())
            .limit(limit)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get recent tracks: {}", err))
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
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_tracks::track_id.eq(track.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!("Failed to get recent tracks: {}", err))
                })?;

            let references: Vec<TrackRefEntity> = schema::track_ref::table
                .filter(schema::track_ref::track_id.eq(track.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!("Failed to get recent tracks: {}", err))
                })?;

            result.push(TrackEntity::convert_to_domain(
                track, album, artists, references,
            ));
        }

        Ok(result)
    }

    fn get_pending_validations(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>> {
        let tracks: Vec<TrackEntity> = schema::track::table
            .filter(schema::track::needs_validation.eq(true))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get pending validations: {}",
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
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_tracks::track_id.eq(track.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get pending validations: {}",
                        err
                    ))
                })?;

            let references: Vec<TrackRefEntity> = schema::track_ref::table
                .filter(schema::track_ref::track_id.eq(track.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get pending validations: {}",
                        err
                    ))
                })?;

            result.push(TrackEntity::convert_to_domain(
                track, album, artists, references,
            ));
        }

        Ok(result)
    }

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Track> {
        let track_ref = schema::track_ref::table
            .filter(schema::track_ref::external_url.eq(url))
            .first::<TrackRefEntity>(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get resource by url: {}", err))
            })?;

        self.get_by_id(conn, track_ref.track_id)
    }

    fn create_references(
        &self,
        conn: &mut SqliteConnection,
        track_id: i32,
        references: &[Reference],
    ) -> SoundomeResult<()> {
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

    fn set_references(
        &self,
        conn: &mut SqliteConnection,
        track_id: i32,
        references: &[Reference],
    ) -> SoundomeResult<()> {
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
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to load track references: {}", err))
            })?;

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
            .left_join(
                schema::album::table.on(schema::album::id.nullable().eq(schema::track::album_id)),
            )
            .filter(schema::track::id.eq(id))
            .first(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get resource by id: {}", err))
            })?;

        let artists: Vec<ArtistEntity> = schema::artist_tracks::table
            .inner_join(
                schema::artist::table.on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
            )
            .filter(schema::artist_tracks::track_id.eq(track.id))
            .select(schema::artist::all_columns)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get resource by id: {}", err))
            })?;

        let references: Vec<TrackRefEntity> = schema::track_ref::table
            .filter(schema::track_ref::track_id.eq(track.id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get resource by id: {}", err))
            })?;

        Ok(TrackEntity::convert_to_domain(
            track, album, artists, references,
        ))
    }

    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>> {
        let tracks: Vec<TrackEntity> = schema::track::table.load(conn).map_err(|err| {
            shared::errors::Error::Database(format!("Failed to get all resources: {}", err))
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
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_tracks::track_id.eq(track.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!("Failed to get all resources: {}", err))
                })?;

            let references: Vec<TrackRefEntity> = schema::track_ref::table
                .filter(schema::track_ref::track_id.eq(track.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!("Failed to get all resources: {}", err))
                })?;

            result.push(TrackEntity::convert_to_domain(
                track, album, artists, references,
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
                shared::errors::Error::Database(format!("Failed to create resource: {}", err))
            })?;

        Ok(TrackEntity::convert_to_domain(
            inserted_track,
            None,
            vec![],
            vec![],
        ))
    }

    fn update(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        updated_track: &Track,
    ) -> SoundomeResult<Track> {
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
                shared::errors::Error::Database(format!("Failed to update resource: {}", err))
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
                (
                    schema::track_ref::table,
                    schema::track_ref::track_id,
                    "Failed to delete associated track references"
                ),
                (
                    schema::artist_tracks::table,
                    schema::artist_tracks::track_id,
                    "Failed to delete associated artist-track relationships"
                ),
                (
                    schema::track::table,
                    schema::track::id,
                    "Failed to delete resource"
                ),
            ]
        )?;
        Ok(())
    }

    fn count(&self, conn: &mut SqliteConnection) -> SoundomeResult<i64> {
        schema::track::table
            .count()
            .get_result(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to count tracks: {}", err))
            })
    }

    fn count_pending_validations(&self, conn: &mut SqliteConnection) -> SoundomeResult<i64> {
        schema::track::table
            .filter(schema::track::needs_validation.eq(true))
            .count()
            .get_result(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to count pending validations: {}",
                    err
                ))
            })
    }

    fn get_by_soundome_id(
        &self,
        conn: &mut SqliteConnection,
        soundome_id: &str,
    ) -> SoundomeResult<Option<Track>> {
        let track: Option<TrackEntity> = schema::track::table
            .filter(schema::track::soundome_id.eq(soundome_id))
            .first::<TrackEntity>(conn)
            .optional()
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get track by soundome_id: {}",
                    err
                ))
            })?;

        let Some(track) = track else {
            return Ok(None);
        };

        let album = if let Some(album_id) = track.album_id {
            schema::album::table
                .filter(schema::album::id.eq(album_id))
                .first::<AlbumEntity>(conn)
                .ok()
        } else {
            None
        };

        let artists: Vec<ArtistEntity> = schema::artist_tracks::table
            .inner_join(
                schema::artist::table.on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
            )
            .filter(schema::artist_tracks::track_id.eq(track.id))
            .select(schema::artist::all_columns)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get artists for track by soundome_id: {}",
                    err
                ))
            })?;

        let references: Vec<TrackRefEntity> = schema::track_ref::table
            .filter(schema::track_ref::track_id.eq(track.id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get references for track by soundome_id: {}",
                    err
                ))
            })?;

        Ok(Some(TrackEntity::convert_to_domain(
            track, album, artists, references,
        )))
    }

    fn get_all_finalized(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>> {
        let tracks: Vec<TrackEntity> = schema::track::table
            .filter(schema::track::file_path.is_not_null())
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get finalized tracks: {}", err))
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
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_tracks::track_id.eq(track.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get finalized tracks: {}",
                        err
                    ))
                })?;

            let references: Vec<TrackRefEntity> = schema::track_ref::table
                .filter(schema::track_ref::track_id.eq(track.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get finalized tracks: {}",
                        err
                    ))
                })?;

            result.push(TrackEntity::convert_to_domain(
                track, album, artists, references,
            ));
        }

        Ok(result)
    }

    fn delete_reference(&self, conn: &mut SqliteConnection, ref_id: i32) -> SoundomeResult<()> {
        diesel::delete(schema::track_ref::table.filter(schema::track_ref::id.eq(ref_id)))
            .execute(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to delete track reference {}: {}",
                    ref_id, err
                ))
            })?;
        Ok(())
    }

    // =================================================================================
    // Pagination / search / sort (library UI)
    // =================================================================================

    fn get_page(&self, conn: &mut SqliteConnection, q: TrackQuery) -> SoundomeResult<Page<Track>> {
        // Sub-query: track ids whose linked artist name matches the search pattern.
        let build_artist_match = |term: &str| {
            let like = format!("%{}%", term);
            schema::artist_tracks::table
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist::name.like(like))
                .select(schema::artist_tracks::track_id)
        };

        // -- total count matching filters (independent of sort/order) --
        let total: i64 = {
            let mut count_q = schema::track::table.into_boxed();
            if let Some(nv) = q.needs_validation {
                count_q = count_q.filter(schema::track::needs_validation.eq(nv));
            }
            if let Some(term) = &q.search {
                let like = format!("%{}%", term);
                count_q = count_q.filter(
                    schema::track::title
                        .like(like)
                        .or(schema::track::id.eq_any(build_artist_match(term))),
                );
            }
            count_q.count().get_result(conn).map_err(|err| {
                shared::errors::Error::Database(format!("Failed to count tracks: {}", err))
            })?
        };

        // -- ids for the requested page, in the requested order --
        let ids: Vec<i32> = match q.sort_by {
            TrackSortBy::Artist => self.ordered_track_ids_by_artist(
                conn,
                q.needs_validation,
                q.search.as_deref(),
                q.sort_dir,
                q.limit,
                q.offset,
            )?,
            TrackSortBy::Album => {
                let mut query = schema::track::table
                    .left_join(
                        schema::album::table
                            .on(schema::album::id.nullable().eq(schema::track::album_id)),
                    )
                    .into_boxed();
                if let Some(nv) = q.needs_validation {
                    query = query.filter(schema::track::needs_validation.eq(nv));
                }
                if let Some(term) = &q.search {
                    let like = format!("%{}%", term);
                    query = query.filter(
                        schema::track::title
                            .like(like)
                            .or(schema::track::id.eq_any(build_artist_match(term))),
                    );
                }
                query = match q.sort_dir {
                    SortDir::Asc => query.order(schema::album::title.asc()),
                    SortDir::Desc => query.order(schema::album::title.desc()),
                };
                query
                    .select(schema::track::id)
                    .limit(q.limit)
                    .offset(q.offset)
                    .load::<i32>(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to page tracks by album: {}",
                            err
                        ))
                    })?
            }
            TrackSortBy::Title | TrackSortBy::Date | TrackSortBy::Duration => {
                let mut query = schema::track::table.into_boxed();
                if let Some(nv) = q.needs_validation {
                    query = query.filter(schema::track::needs_validation.eq(nv));
                }
                if let Some(term) = &q.search {
                    let like = format!("%{}%", term);
                    query = query.filter(
                        schema::track::title
                            .like(like)
                            .or(schema::track::id.eq_any(build_artist_match(term))),
                    );
                }
                query = match (q.sort_by, q.sort_dir) {
                    (TrackSortBy::Title, SortDir::Asc) => query.order(schema::track::title.asc()),
                    (TrackSortBy::Title, SortDir::Desc) => query.order(schema::track::title.desc()),
                    (TrackSortBy::Date, SortDir::Asc) => query.order(schema::track::date.asc()),
                    (TrackSortBy::Date, SortDir::Desc) => query.order(schema::track::date.desc()),
                    (TrackSortBy::Duration, SortDir::Asc) => {
                        query.order(schema::track::duration.asc())
                    }
                    (TrackSortBy::Duration, SortDir::Desc) => {
                        query.order(schema::track::duration.desc())
                    }
                    _ => unreachable!("Artist/Album sorts are handled in dedicated branches"),
                };
                query
                    .select(schema::track::id)
                    .limit(q.limit)
                    .offset(q.offset)
                    .load::<i32>(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!("Failed to page tracks: {}", err))
                    })?
            }
        };

        let items = self.hydrate_tracks_by_ids(conn, &ids)?;
        Ok(Page { items, total })
    }

    fn get_by_artist(
        &self,
        conn: &mut SqliteConnection,
        artist_id: i32,
    ) -> SoundomeResult<Vec<Track>> {
        let ids: Vec<i32> = schema::artist_tracks::table
            .filter(schema::artist_tracks::artist_id.eq(artist_id))
            .select(schema::artist_tracks::track_id)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to load tracks for artist {}: {}",
                    artist_id, err
                ))
            })?;
        self.hydrate_tracks_by_ids(conn, &ids)
    }

    fn get_by_album(
        &self,
        conn: &mut SqliteConnection,
        album_id: i32,
    ) -> SoundomeResult<Vec<Track>> {
        let ids: Vec<i32> = schema::track::table
            .filter(schema::track::album_id.eq(album_id))
            .select(schema::track::id)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to load tracks for album {}: {}",
                    album_id, err
                ))
            })?;
        self.hydrate_tracks_by_ids(conn, &ids)
    }

    fn merge_into(
        &self,
        conn: &mut SqliteConnection,
        source_ids: &[i32],
        target_id: i32,
        merged_track: &Track,
    ) -> SoundomeResult<()> {
        use diesel::Connection as _;

        if source_ids.is_empty() || source_ids.contains(&target_id) {
            return Err(shared::errors::Error::Custom(
                "Track merge requires source ids distinct from the target".to_string(),
            ));
        }
        let mut unique_sources = HashSet::with_capacity(source_ids.len());
        if source_ids.iter().any(|id| !unique_sources.insert(*id)) {
            return Err(shared::errors::Error::Custom(
                "Track merge source ids must be unique".to_string(),
            ));
        }

        conn.transaction(|tx| {
            // Apply the quality winner's metadata/path to the surviving row.
            self.update(tx, target_id, merged_track)?;

            // Keep the winner's artist relationships; artist associations from
            // inferior duplicate recordings are not copied onto the survivor.
            diesel::delete(
                schema::artist_tracks::table.filter(schema::artist_tracks::track_id.eq(target_id)),
            )
            .execute(tx)
            .map_err(|e| {
                shared::errors::Error::Database(format!("merge: clear target artists: {e}"))
            })?;
            let mut artist_ids = HashSet::new();
            for artist_id in merged_track.artists.iter().filter_map(|artist| artist.id) {
                if artist_ids.insert(artist_id) {
                    diesel::insert_into(schema::artist_tracks::table)
                        .values(ArtistTrackEntity {
                            track_id: target_id,
                            artist_id,
                        })
                        .execute(tx)
                        .map_err(|e| {
                            shared::errors::Error::Database(format!(
                                "merge: set target artists: {e}"
                            ))
                        })?;
                }
            }

            // Preserve the union of playlist memberships. When two duplicate
            // tracks occur in the same playlist, keep the target's existing
            // position; otherwise keep the first source position encountered.
            let all_ids: Vec<i32> = std::iter::once(target_id)
                .chain(source_ids.iter().copied())
                .collect();
            let playlist_rows: Vec<(i32, i32, Option<i32>)> = schema::playlist_tracks::table
                .filter(schema::playlist_tracks::track_id.eq_any(&all_ids))
                .select((
                    schema::playlist_tracks::track_id,
                    schema::playlist_tracks::playlist_id,
                    schema::playlist_tracks::position,
                ))
                .load(tx)
                .map_err(|e| {
                    shared::errors::Error::Database(format!("merge: load playlist links: {e}"))
                })?;
            let mut playlist_positions: HashMap<i32, Option<i32>> = HashMap::new();
            for (track_id, playlist_id, position) in &playlist_rows {
                if *track_id == target_id {
                    playlist_positions.insert(*playlist_id, *position);
                }
            }
            for (_, playlist_id, position) in &playlist_rows {
                playlist_positions.entry(*playlist_id).or_insert(*position);
            }
            for (playlist_id, position) in playlist_positions {
                diesel::insert_or_ignore_into(schema::playlist_tracks::table)
                    .values((
                        schema::playlist_tracks::track_id.eq(target_id),
                        schema::playlist_tracks::playlist_id.eq(playlist_id),
                        schema::playlist_tracks::position.eq(position),
                    ))
                    .execute(tx)
                    .map_err(|e| {
                        shared::errors::Error::Database(format!(
                            "merge: preserve playlist link: {e}"
                        ))
                    })?;
            }

            // `track_genres` is currently unused by Soundome's model, but keep
            // any existing rows rather than dropping them during a merge.
            let genre_ids: Vec<i32> = schema::track_genres::table
                .filter(schema::track_genres::track_id.eq_any(&all_ids))
                .select(schema::track_genres::genre_id)
                .distinct()
                .load(tx)
                .map_err(|e| {
                    shared::errors::Error::Database(format!("merge: load track genres: {e}"))
                })?;
            for genre_id in genre_ids {
                diesel::insert_or_ignore_into(schema::track_genres::table)
                    .values((
                        schema::track_genres::track_id.eq(target_id),
                        schema::track_genres::genre_id.eq(genre_id),
                    ))
                    .execute(tx)
                    .map_err(|e| {
                        shared::errors::Error::Database(format!("merge: preserve track genre: {e}"))
                    })?;
            }

            // References supplied by the domain service already follow the
            // Source/Provider replacement and Metadata/Reference merge rules.
            self.set_references(tx, target_id, &merged_track.references)?;

            diesel::delete(
                schema::dedup_ignore::table
                    .filter(schema::dedup_ignore::entity_type.eq("track"))
                    .filter(
                        schema::dedup_ignore::id_a
                            .eq_any(source_ids)
                            .or(schema::dedup_ignore::id_b.eq_any(source_ids)),
                    ),
            )
            .execute(tx)
            .map_err(|e| {
                shared::errors::Error::Database(format!("merge: delete stale track ignores: {e}"))
            })?;

            for &source_id in source_ids {
                diesel::delete(
                    schema::playlist_tracks::table
                        .filter(schema::playlist_tracks::track_id.eq(source_id)),
                )
                .execute(tx)
                .map_err(|e| {
                    shared::errors::Error::Database(format!(
                        "merge: remove source playlist links: {e}"
                    ))
                })?;
                diesel::delete(
                    schema::track_genres::table
                        .filter(schema::track_genres::track_id.eq(source_id)),
                )
                .execute(tx)
                .map_err(|e| {
                    shared::errors::Error::Database(format!("merge: remove source genres: {e}"))
                })?;
                self.delete(tx, source_id)?;
            }

            Ok(())
        })
    }
}
