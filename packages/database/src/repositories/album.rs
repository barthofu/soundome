// basic CRUD operations

use domain::ports::repositories::AlbumRepository;

use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::{models::{Album, Reference}, types::SoundomeResult};

use crate::{
    delete_with_relations, entities::{
        AlbumEntity, AlbumRefEntity, ArtistEntity, NewAlbumEntity, NewAlbumRefEntity, UpdateAlbumEntity
    }, schema
};

use crate::diesel::Connection;

pub struct DieselAlbumRepository {}

impl DieselAlbumRepository {
    pub fn new() -> Self {
        Self {}
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
                shared::errors::Error::Database(format!(
                    "Failed to get resource by url: {}",
                    err
                ))
            })?;

        self.get_by_id(conn, album_ref.album_id)
    }

    fn create_references(&self, conn: &mut SqliteConnection, album_id: i32, references: &[Reference]) -> SoundomeResult<()> {
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

    fn set_references(&self, conn: &mut SqliteConnection, album_id: i32, references: &[Reference]) -> SoundomeResult<()> {
        // Merge semantics: keep existing rows (and their ids), only insert missing refs.
        if references.is_empty() {
            return Ok(());
        }

        let existing: Vec<AlbumRefEntity> = schema::album_ref::table
            .filter(schema::album_ref::album_id.eq(album_id))
            .load(conn)
            .map_err(|err| shared::errors::Error::Database(format!("Failed to load album references: {}", err)))?;

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

    fn create_or_ignore(&self, conn: &mut SqliteConnection, album: &Album) -> SoundomeResult<Album> {
        // If album already has an ID, return it as-is (ignore creation)
        if album.id.is_some() {
            return Ok(album.clone());
        }
        // Otherwise, create the album and its references
        let created_album = self.create(conn, album)?;
        let album_id = created_album.id.unwrap();
        // Create album references
        self.create_references(conn, album_id, &album.references)?;
        // Return the created album with references
        self.get_by_id(conn, album_id)
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
        let albums: Vec<AlbumEntity> = schema::album::table
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get all albums: {}",
                    err
                ))
            })?;

        let mut result = Vec::new();
        for album in albums {
            let artists: Vec<ArtistEntity> = schema::artist_albums::table
                .inner_join(schema::artist::table.on(schema::artist_albums::artist_id.eq(schema::artist::id)))
                .filter(schema::artist_albums::album_id.eq(album.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get album artists: {}",
                        err
                    ))
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
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let artists: Vec<ArtistEntity> = schema::artist_albums::table
            .inner_join(schema::artist::table.on(schema::artist_albums::artist_id.eq(schema::artist::id)))
            .filter(schema::artist_albums::album_id.eq(album.id))
            .select(schema::artist::all_columns)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let references: Vec<AlbumRefEntity> = schema::album_ref::table
            .filter(schema::album_ref::album_id.eq(album.id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        Ok(AlbumEntity::convert_to_domain(
            album,
            artists,
            references,
        ))
    }

    fn create(
        &self,
        conn: &mut SqliteConnection,
        new_album: &Album,
    ) -> SoundomeResult<Album> {

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
                shared::errors::Error::Database(format!(
                    "Failed to create resource: {}",
                    err
                ))
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
                shared::errors::Error::Database(format!(
                    "Failed to update resource: {}",
                    err
                ))
            })?;

        self.get_by_id(conn, id)
    }

    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        
        delete_with_relations!(
            conn,
            id,
            [
                (schema::album_ref::table, schema::album_ref::album_id, "Failed to delete album references"),
                (schema::artist_albums::table, schema::artist_albums::album_id, "Failed to delete artist-album relations"),
                (schema::album::table, schema::album::id, "Failed to delete resource"),
            ]
        )?;
        Ok(())

    }
}
