use domain::ports::repositories::ArtistRepository;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
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

    fn create_or_ignore(&self, conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Artist> {
        // If artist already has an ID, return it as-is (ignore creation)
        if artist.id.is_some() {
            return Ok(artist.clone());
        }
        // Otherwise, create the artist and ses références
        let created_artist = self.create(conn, artist)?;
        let artist_id = created_artist.id.unwrap();
        // Create artist references
        self.create_references(conn, artist_id, &artist.references)?;
        // Return the created artist with references
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

}
