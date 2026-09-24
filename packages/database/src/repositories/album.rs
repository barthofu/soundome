// basic CRUD operations

use domain::ports::repositories::{
    AlbumQuery, AlbumRepository, AlbumSortBy, NamePair, Page, SortDir,
};

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Integer, Nullable, Text};
use diesel::{
    ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection,
};
use shared::{
    models::{Album, Reference},
    types::SoundomeResult,
};

use crate::{
    delete_with_relations,
    entities::{
        AlbumEntity, AlbumRefEntity, ArtistEntity, NewAlbumEntity, NewAlbumRefEntity,
        UpdateAlbumEntity,
    },
    schema,
};

use crate::diesel::Connection;

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = Integer)]
    id: i32,
}

#[derive(Default)]
pub struct DieselAlbumRepository {}

impl DieselAlbumRepository {
    pub fn new() -> Self {
        Self {}
    }

    /// Hydrate a page of albums (artists + references) given an
    /// already-ordered list of ids, preserving that order.
    fn hydrate_albums_by_ids(
        &self,
        conn: &mut SqliteConnection,
        ids: &[i32],
    ) -> SoundomeResult<Vec<Album>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let entities: Vec<AlbumEntity> = schema::album::table
            .filter(schema::album::id.eq_any(ids.to_vec()))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to load albums page: {}", err))
            })?;
        let mut by_id: std::collections::HashMap<i32, AlbumEntity> =
            entities.into_iter().map(|e| (e.id, e)).collect();

        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            let Some(album) = by_id.remove(id) else {
                continue;
            };

            let artists: Vec<ArtistEntity> = schema::artist_albums::table
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_albums::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_albums::album_id.eq(album.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to load artists for album page: {}",
                        err
                    ))
                })?;

            let references: Vec<AlbumRefEntity> = schema::album_ref::table
                .filter(schema::album_ref::album_id.eq(album.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to load references for album page: {}",
                        err
                    ))
                })?;

            result.push(AlbumEntity::convert_to_domain(album, artists, references));
        }

        Ok(result)
    }

    /// Order+page album ids by their (first, alphabetically) linked artist name.
    fn ordered_album_ids_by_artist(
        &self,
        conn: &mut SqliteConnection,
        search: Option<&str>,
        dir: SortDir,
        limit: i64,
        offset: i64,
    ) -> SoundomeResult<Vec<i32>> {
        let sql = format!(
            "SELECT a.id AS id \
             FROM album a \
             LEFT JOIN ( \
                 SELECT aa.album_id AS album_id, MIN(ar.name) AS artist_name \
                 FROM artist_albums aa JOIN artist ar ON aa.artist_id = ar.id \
                 GROUP BY aa.album_id \
             ) ta ON ta.album_id = a.id \
             WHERE (? IS NULL OR a.title LIKE ? OR a.id IN ( \
                     SELECT aa2.album_id FROM artist_albums aa2 \
                     JOIN artist ar2 ON aa2.artist_id = ar2.id \
                     WHERE ar2.name LIKE ? \
                   )) \
             ORDER BY ta.artist_name {} \
             LIMIT ? OFFSET ?",
            dir.as_sql()
        );

        let pattern = search.map(|s| format!("%{}%", s));

        let rows: Vec<IdRow> = diesel::sql_query(sql)
            .bind::<Nullable<Text>, _>(pattern.clone())
            .bind::<Nullable<Text>, _>(pattern.clone())
            .bind::<Nullable<Text>, _>(pattern)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to sort albums by artist: {}", err))
            })?;

        Ok(rows.into_iter().map(|r| r.id).collect())
    }

    /// Order+page album ids by their linked track count.
    fn ordered_album_ids_by_track_count(
        &self,
        conn: &mut SqliteConnection,
        search: Option<&str>,
        dir: SortDir,
        limit: i64,
        offset: i64,
    ) -> SoundomeResult<Vec<i32>> {
        let sql = format!(
            "SELECT a.id AS id \
             FROM album a \
             LEFT JOIN track t ON t.album_id = a.id \
             WHERE (? IS NULL OR a.title LIKE ? OR a.id IN ( \
                     SELECT aa.album_id FROM artist_albums aa \
                     JOIN artist ar ON aa.artist_id = ar.id \
                     WHERE ar.name LIKE ? \
                   )) \
             GROUP BY a.id \
             ORDER BY COUNT(t.id) {}, a.title ASC \
             LIMIT ? OFFSET ?",
            dir.as_sql()
        );

        let pattern = search.map(|s| format!("%{}%", s));

        let rows: Vec<IdRow> = diesel::sql_query(sql)
            .bind::<Nullable<Text>, _>(pattern.clone())
            .bind::<Nullable<Text>, _>(pattern.clone())
            .bind::<Nullable<Text>, _>(pattern)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to sort albums by track count: {}",
                    err
                ))
            })?;

        Ok(rows.into_iter().map(|r| r.id).collect())
    }

    /// Looks up an existing album by matching any of `references` against stored
    /// `album_ref` rows on `(platform, external_id)` or `(platform, external_url)`.
    /// Used to dedupe by durable platform identity before falling back to a
    /// title-based match. See `DieselArtistRepository::find_artist_id_by_any_reference`.
    fn find_album_id_by_any_reference(
        &self,
        conn: &mut SqliteConnection,
        references: &[Reference],
    ) -> SoundomeResult<Option<i32>> {
        for reference in references {
            if reference.external_id.is_none() && reference.external_url.is_none() {
                continue;
            }
            let platform = reference.platform.as_ref().to_string().to_lowercase();

            if let Some(external_id) = &reference.external_id {
                let found: Option<AlbumRefEntity> = schema::album_ref::table
                    .filter(schema::album_ref::platform.eq(&platform))
                    .filter(schema::album_ref::external_id.eq(external_id))
                    .first(conn)
                    .optional()
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to look up album by reference external_id: {}",
                            err
                        ))
                    })?;
                if let Some(entity) = found {
                    return Ok(Some(entity.album_id));
                }
            }

            if let Some(external_url) = &reference.external_url {
                let found: Option<AlbumRefEntity> = schema::album_ref::table
                    .filter(schema::album_ref::platform.eq(&platform))
                    .filter(schema::album_ref::external_url.eq(external_url))
                    .first(conn)
                    .optional()
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to look up album by reference external_url: {}",
                            err
                        ))
                    })?;
                if let Some(entity) = found {
                    return Ok(Some(entity.album_id));
                }
            }
        }
        Ok(None)
    }
}

impl AlbumRepository for DieselAlbumRepository {
    // =================================================================================
    // Custom
    // =================================================================================

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Album> {
        let album_ref = schema::album_ref::table
            .filter(schema::album_ref::external_url.eq(url))
            .first::<AlbumRefEntity>(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get resource by url: {}", err))
            })?;

        self.get_by_id(conn, album_ref.album_id)
    }

    fn create_references(
        &self,
        conn: &mut SqliteConnection,
        album_id: i32,
        references: &[Reference],
    ) -> SoundomeResult<()> {
        for reference in references {
            let new_album_ref = NewAlbumRefEntity::convert_from_domain(reference, album_id);

            diesel::insert_into(schema::album_ref::table)
                .values(&new_album_ref)
                .execute(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to create album reference: {}",
                        err
                    ))
                })?;
        }
        Ok(())
    }

    fn set_references(
        &self,
        conn: &mut SqliteConnection,
        album_id: i32,
        references: &[Reference],
    ) -> SoundomeResult<()> {
        // Merge semantics: keep existing rows (and their ids), only insert missing refs.
        if references.is_empty() {
            return Ok(());
        }

        let existing: Vec<AlbumRefEntity> = schema::album_ref::table
            .filter(schema::album_ref::album_id.eq(album_id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to load album references: {}", err))
            })?;

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
                let new_album_ref = NewAlbumRefEntity::convert_from_domain(reference, album_id);
                diesel::insert_into(schema::album_ref::table)
                    .values(&new_album_ref)
                    .execute(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!(
                            "Failed to create album reference: {}",
                            err
                        ))
                    })?;
            }
        }

        Ok(())
    }

    fn create_or_ignore(
        &self,
        conn: &mut SqliteConnection,
        album: &Album,
    ) -> SoundomeResult<Album> {
        // If album already has an ID, return it as-is
        if let Some(id) = album.id {
            return self.get_by_id(conn, id);
        }
        // Reference fast path: a matching (platform, external_id/external_url) is a
        // stronger identity signal than the title and takes priority over
        // title-based matching below (see artist repo for rationale).
        if let Some(existing_id) = self.find_album_id_by_any_reference(conn, &album.references)? {
            self.set_references(conn, existing_id, &album.references)?;
            return self.get_by_id(conn, existing_id);
        }
        // Exact-title fast path
        let exact: Option<AlbumEntity> = schema::album::table
            .filter(schema::album::title.eq(&album.title))
            .first(conn)
            .optional()
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to look up album: {}", err))
            })?;
        if let Some(entity) = exact {
            // Merge references when finding an existing album by exact title
            self.set_references(conn, entity.id, &album.references)?;
            return self.get_by_id(conn, entity.id);
        }
        // Case-insensitive fallback (Unicode-safe: compare lowercased in Rust)
        let title_lower = album.title.to_lowercase();
        let all: Vec<AlbumEntity> = schema::album::table.load(conn).map_err(|err| {
            shared::errors::Error::Database(format!("Failed to load albums for dedup: {}", err))
        })?;
        if let Some(entity) = all
            .into_iter()
            .find(|e| e.title.to_lowercase() == title_lower)
        {
            // Merge references when finding an existing album by case-insensitive title
            self.set_references(conn, entity.id, &album.references)?;
            return self.get_by_id(conn, entity.id);
        }
        // Not found: create the album and its references
        let created_album = self.create(conn, album)?;
        let album_id = created_album.id.unwrap();
        self.create_references(conn, album_id, &album.references)?;
        self.get_by_id(conn, album_id)
    }

    fn find_by_title_and_artists(
        &self,
        conn: &mut SqliteConnection,
        title: &str,
        artist_names: &[String],
    ) -> SoundomeResult<Option<Album>> {
        if artist_names.is_empty() {
            // No artist hint — fall back to title-only (safe: caller owns the flow)
            let entity: Option<AlbumEntity> = schema::album::table
                .filter(schema::album::title.eq(title))
                .first(conn)
                .optional()
                .map_err(|e| shared::errors::Error::Database(e.to_string()))?;
            return match entity {
                Some(e) => self.get_by_id(conn, e.id).map(Some),
                None => Ok(None),
            };
        }

        let title_lower = title.to_lowercase();
        let artist_names_lower: Vec<String> =
            artist_names.iter().map(|s| s.to_lowercase()).collect();

        // Load all albums whose title matches (exact or case-insensitive).
        let candidates: Vec<AlbumEntity> = schema::album::table
            .filter(schema::album::title.eq(title))
            .load(conn)
            .or_else(|_| {
                // If exact-case fails, load all and filter in Rust for Unicode safety.
                schema::album::table.load::<AlbumEntity>(conn).map(|all| {
                    all.into_iter()
                        .filter(|e| e.title.to_lowercase() == title_lower)
                        .collect()
                })
            })
            .map_err(|e| shared::errors::Error::Database(e.to_string()))?;

        for candidate in candidates {
            // Load this album's artists and check for a name overlap.
            let artists: Vec<ArtistEntity> = schema::artist_albums::table
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_albums::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_albums::album_id.eq(candidate.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .unwrap_or_default();

            let has_matching_artist = artists.iter().any(|a| {
                let a_lower = a.name.to_lowercase();
                artist_names_lower.contains(&a_lower)
            });

            if has_matching_artist {
                return self.get_by_id(conn, candidate.id).map(Some);
            }
        }

        Ok(None)
    }

    // fn find_by_unique_fields(&self, conn: &mut SqliteConnection, album: &Album) -> SoundomeResult<Option<Album>> {
    //     use diesel::prelude::*;
    //     use crate::schema;
    //     use crate::schema::album::dsl::*;
    //     let mut query = album.into_boxed();
    //     query = query.filter(title.eq(&album.title));
    //     if let Some(ref d) = album.date {
    //         query = query.filter(date.eq(d));
    //     }
    //     let found: Option<AlbumEntity> = query
    //         .first::<AlbumEntity>(conn)
    //         .optional()
    //         .map_err(|err| shared::errors::Error::Database(format!("Failed to find album by unique fields: {}", err)))?;
    //     if let Some(entity) = found {
    //         // Charger les artistes et références si besoin
    //         let artists: Vec<ArtistEntity> = schema::artist_albums::table
    //             .inner_join(schema::artist::table.on(schema::artist_albums::artist_id.eq(schema::artist::id)))
    //             .filter(schema::artist_albums::album_id.eq(entity.id))
    //             .select(schema::artist::all_columns)
    //             .load(conn)
    //             .unwrap_or_default();
    //         let references: Vec<AlbumRefEntity> = schema::album_ref::table
    //             .filter(schema::album_ref::album_id.eq(entity.id))
    //             .load(conn)
    //             .unwrap_or_default();
    //         Ok(Some(AlbumEntity::convert_to_domain(entity, artists, references)))
    //     } else {
    //         Ok(None)
    //     }
    // }

    // =================================================================================
    // CRUD
    // =================================================================================

    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Album>> {
        let albums: Vec<AlbumEntity> = schema::album::table.load(conn).map_err(|err| {
            shared::errors::Error::Database(format!("Failed to get all albums: {}", err))
        })?;

        let mut result = Vec::new();
        for album in albums {
            let artists: Vec<ArtistEntity> = schema::artist_albums::table
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_albums::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_albums::album_id.eq(album.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!("Failed to get album artists: {}", err))
                })?;

            let references: Vec<AlbumRefEntity> = schema::album_ref::table
                .filter(schema::album_ref::album_id.eq(album.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get album references: {}",
                        err
                    ))
                })?;

            result.push(AlbumEntity::convert_to_domain(album, artists, references));
        }

        Ok(result)
    }

    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Album> {
        let album: AlbumEntity = schema::album::table
            .filter(schema::album::id.eq(id))
            .first(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get resource by id: {}", err))
            })?;

        let artists: Vec<ArtistEntity> = schema::artist_albums::table
            .inner_join(
                schema::artist::table.on(schema::artist_albums::artist_id.eq(schema::artist::id)),
            )
            .filter(schema::artist_albums::album_id.eq(album.id))
            .select(schema::artist::all_columns)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get resource by id: {}", err))
            })?;

        let references: Vec<AlbumRefEntity> = schema::album_ref::table
            .filter(schema::album_ref::album_id.eq(album.id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to get resource by id: {}", err))
            })?;

        Ok(AlbumEntity::convert_to_domain(album, artists, references))
    }

    fn create(&self, conn: &mut SqliteConnection, new_album: &Album) -> SoundomeResult<Album> {
        let new_album_entity = NewAlbumEntity::convert_from_domain(new_album);
        let inserted_album = diesel::insert_into(schema::album::table)
            .values(&new_album_entity)
            .execute(conn)
            .and_then(|_| {
                schema::album::table
                    .order(schema::album::id.desc())
                    .first::<AlbumEntity>(conn)
            })
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to create resource: {}", err))
            })?;

        Ok(AlbumEntity::convert_to_domain(
            inserted_album,
            vec![],
            vec![],
        ))
    }

    fn update(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        updated_album: &Album,
    ) -> SoundomeResult<Album> {
        let updated_album_entity = UpdateAlbumEntity::convert_from_domain(updated_album);
        diesel::update(schema::album::table)
            .filter(schema::album::id.eq(id))
            .set(&updated_album_entity)
            .execute(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to update resource: {}", err))
            })?;

        self.get_by_id(conn, id)
    }

    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        delete_with_relations!(
            conn,
            id,
            [
                (
                    schema::album_ref::table,
                    schema::album_ref::album_id,
                    "Failed to delete album references"
                ),
                (
                    schema::artist_albums::table,
                    schema::artist_albums::album_id,
                    "Failed to delete artist-album relations"
                ),
                (
                    schema::album::table,
                    schema::album::id,
                    "Failed to delete resource"
                ),
            ]
        )?;
        Ok(())
    }

    fn count(&self, conn: &mut SqliteConnection) -> SoundomeResult<i64> {
        schema::album::table
            .count()
            .get_result(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to count albums: {}", err))
            })
    }

    fn count_tracks(&self, conn: &mut SqliteConnection, album_id: i32) -> SoundomeResult<i64> {
        schema::track::table
            .filter(schema::track::album_id.eq(album_id))
            .count()
            .get_result(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to count tracks for album {}: {}",
                    album_id, err
                ))
            })
    }

    fn delete_reference(&self, conn: &mut SqliteConnection, ref_id: i32) -> SoundomeResult<()> {
        diesel::delete(schema::album_ref::table.filter(schema::album_ref::id.eq(ref_id)))
            .execute(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to delete album reference {}: {}",
                    ref_id, err
                ))
            })?;
        Ok(())
    }

    fn merge_into(
        &self,
        conn: &mut SqliteConnection,
        source_ids: &[i32],
        target_id: i32,
    ) -> SoundomeResult<()> {
        conn.transaction(|conn| {
            // --- Re-point tracks (direct FK on track.album_id) --------------------------
            for &src in source_ids {
                diesel::update(schema::track::table.filter(schema::track::album_id.eq(src)))
                    .set(schema::track::album_id.eq(target_id))
                    .execute(conn)
                    .map_err(|e| {
                        shared::errors::Error::Database(format!(
                            "merge: repoint track.album_id: {e}"
                        ))
                    })?;
            }

            // --- Re-point artist_albums ---------------------------------------------------
            // PK is (album_id, artist_id); avoid inserting a duplicate combo for the target.
            let target_artist_ids: Vec<i32> = schema::artist_albums::table
                .filter(schema::artist_albums::album_id.eq(target_id))
                .select(schema::artist_albums::artist_id)
                .load(conn)
                .map_err(|e| {
                    shared::errors::Error::Database(format!(
                        "merge: load target album artists: {e}"
                    ))
                })?;

            for &src in source_ids {
                let src_artist_ids: Vec<i32> = schema::artist_albums::table
                    .filter(schema::artist_albums::album_id.eq(src))
                    .select(schema::artist_albums::artist_id)
                    .load(conn)
                    .map_err(|e| {
                        shared::errors::Error::Database(format!(
                            "merge: load source album artists: {e}"
                        ))
                    })?;

                for artist_id in src_artist_ids {
                    if !target_artist_ids.contains(&artist_id) {
                        diesel::insert_into(schema::artist_albums::table)
                            .values(crate::entities::ArtistAlbumEntity {
                                album_id: target_id,
                                artist_id,
                            })
                            .execute(conn)
                            .map_err(|e| {
                                shared::errors::Error::Database(format!(
                                    "merge: insert artist_album: {e}"
                                ))
                            })?;
                    }
                }
                diesel::delete(
                    schema::artist_albums::table.filter(schema::artist_albums::album_id.eq(src)),
                )
                .execute(conn)
                .map_err(|e| {
                    shared::errors::Error::Database(format!(
                        "merge: delete source artist_albums: {e}"
                    ))
                })?;
            }

            // --- Move album_refs -----------------------------------------------------------
            for &src in source_ids {
                diesel::update(
                    schema::album_ref::table.filter(schema::album_ref::album_id.eq(src)),
                )
                .set(schema::album_ref::album_id.eq(target_id))
                .execute(conn)
                .map_err(|e| {
                    shared::errors::Error::Database(format!("merge: move album_ref: {e}"))
                })?;
            }

            // --- Delete source albums --------------------------------------------------------
            diesel::delete(
                schema::dedup_ignore::table
                    .filter(schema::dedup_ignore::entity_type.eq("album"))
                    .filter(
                        schema::dedup_ignore::id_a
                            .eq_any(source_ids)
                            .or(schema::dedup_ignore::id_b.eq_any(source_ids)),
                    ),
            )
            .execute(conn)
            .map_err(|e| {
                shared::errors::Error::Database(format!("merge: delete stale album ignores: {e}"))
            })?;

            for &src in source_ids {
                diesel::delete(schema::album::table.filter(schema::album::id.eq(src)))
                    .execute(conn)
                    .map_err(|e| {
                        shared::errors::Error::Database(format!("merge: delete source album: {e}"))
                    })?;
            }

            Ok(())
        })
    }

    // =================================================================================
    // Pagination / search / sort (library UI)
    // =================================================================================

    fn get_page(&self, conn: &mut SqliteConnection, q: AlbumQuery) -> SoundomeResult<Page<Album>> {
        let build_artist_match = |term: &str| {
            let like = format!("%{}%", term);
            schema::artist_albums::table
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_albums::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist::name.like(like))
                .select(schema::artist_albums::album_id)
        };

        let total: i64 = {
            let mut count_q = schema::album::table.into_boxed();
            if let Some(term) = &q.search {
                let like = format!("%{}%", term);
                count_q = count_q.filter(
                    schema::album::title
                        .like(like)
                        .or(schema::album::id.eq_any(build_artist_match(term))),
                );
            }
            count_q.count().get_result(conn).map_err(|err| {
                shared::errors::Error::Database(format!("Failed to count albums: {}", err))
            })?
        };

        let ids: Vec<i32> = match q.sort_by {
            AlbumSortBy::Artist => self.ordered_album_ids_by_artist(
                conn,
                q.search.as_deref(),
                q.sort_dir,
                q.limit,
                q.offset,
            )?,
            AlbumSortBy::TrackCount => self.ordered_album_ids_by_track_count(
                conn,
                q.search.as_deref(),
                q.sort_dir,
                q.limit,
                q.offset,
            )?,
            AlbumSortBy::Title | AlbumSortBy::Date => {
                let mut query = schema::album::table.into_boxed();
                if let Some(term) = &q.search {
                    let like = format!("%{}%", term);
                    query = query.filter(
                        schema::album::title
                            .like(like)
                            .or(schema::album::id.eq_any(build_artist_match(term))),
                    );
                }
                query = match (q.sort_by, q.sort_dir) {
                    (AlbumSortBy::Title, SortDir::Asc) => query.order(schema::album::title.asc()),
                    (AlbumSortBy::Title, SortDir::Desc) => query.order(schema::album::title.desc()),
                    (AlbumSortBy::Date, SortDir::Asc) => query.order(schema::album::date.asc()),
                    (AlbumSortBy::Date, SortDir::Desc) => query.order(schema::album::date.desc()),
                    _ => unreachable!("Artist/TrackCount sorts are handled in dedicated branches"),
                };
                query
                    .select(schema::album::id)
                    .limit(q.limit)
                    .offset(q.offset)
                    .load::<i32>(conn)
                    .map_err(|err| {
                        shared::errors::Error::Database(format!("Failed to page albums: {}", err))
                    })?
            }
        };

        let items = self.hydrate_albums_by_ids(conn, &ids)?;
        Ok(Page { items, total })
    }

    fn get_names(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<NamePair>> {
        let rows: Vec<(i32, String)> = schema::album::table
            .select((schema::album::id, schema::album::title))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!("Failed to load album names: {}", err))
            })?;
        Ok(rows
            .into_iter()
            .map(|(id, name)| NamePair { id, name })
            .collect())
    }

    fn get_by_artist(
        &self,
        conn: &mut SqliteConnection,
        artist_id: i32,
    ) -> SoundomeResult<Vec<Album>> {
        let ids: Vec<i32> = schema::artist_albums::table
            .filter(schema::artist_albums::artist_id.eq(artist_id))
            .select(schema::artist_albums::album_id)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to load albums for artist {}: {}",
                    artist_id, err
                ))
            })?;
        self.hydrate_albums_by_ids(conn, &ids)
    }
}
