use domain::ports::repositories::ArtistRepository;

use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::types::SoundomeResult;

use crate::{
    entities::{
        ArtistEntity, ArtistRefEntity, ArtistTrackEntity, NewArtistEntity, TrackEntity, UpdateArtistEntity,
        NewArtistRefEntity, ArtistAlbumEntity
    }, macros, schema
};

pub struct DieselArtistRepository {}

impl DieselArtistRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl ArtistRepository for DieselArtistRepository {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<shared::models::Artist> {
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

    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<shared::models::Artist>> {
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

    fn create(&self, conn: &mut SqliteConnection, new_artist: &shared::models::Artist) -> SoundomeResult<shared::models::Artist> {
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

    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_artist: &shared::models::Artist) -> SoundomeResult<shared::models::Artist> {
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

    // =================================================================================
    // Custom
    // =================================================================================

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<shared::models::Artist> {
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

    fn create_references(&self, conn: &mut SqliteConnection, artist_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()> {
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

    fn create_or_ignore(&self, conn: &mut SqliteConnection, artist: &shared::models::Artist) -> SoundomeResult<shared::models::Artist> {
        // If artist already has an ID, return it as-is (ignore creation)
        if artist.id.is_some() {
            return Ok(artist.clone());
        }
        
        // Otherwise, create the artist and its references
        let created_artist = self.create(conn, artist)?;
        let artist_id = created_artist.id.unwrap();
        
        // Create artist references
        self.create_references(conn, artist_id, &artist.references)?;
        
        // Return the created artist with references
        self.get_by_id(conn, artist_id)
    }
}




















// ================================================================================================
// ARCHIVES
// ================================================================================================

// basic CRUD operations

macros::resource::find_one!(artist, ArtistEntity);
macros::resource::find_all!(artist, ArtistEntity);
macros::resource::create!(artist, ArtistEntity, NewArtistEntity);
macros::resource::update!(artist, ArtistEntity, UpdateArtistEntity);
macros::resource::delete!(artist);

// associations

macros::association::many_to_many::get_all_associations!(
    get_tracks,
    ArtistEntity,
    track,
    TrackEntity,
    artist_tracks,
    ArtistTrackEntity,
    track_id,
);

macros::association::many_to_many::create_association!(
    create_track,
    ArtistEntity,
    TrackEntity,
    artist_tracks,
    ArtistTrackEntity,
    track_id,
    artist_id,
);

macros::association::many_to_many::delete_association!(
    delete_track,
    ArtistEntity,
    TrackEntity,
    artist_tracks,
    artist_id,
    track_id,
);

// custom operations

pub fn has_track(conn: &mut SqliteConnection, artist: &ArtistEntity, track: &TrackEntity) -> bool {
    ArtistTrackEntity::belonging_to(&artist)
        .select(crate::schema::artist_tracks::track_id)
        .filter(crate::schema::artist_tracks::track_id.eq(track.id))
        .first::<i32>(conn)
        .is_ok()
}

// ================================================================================================
// Artist Ref
// ================================================================================================

pub mod artist_ref {
    use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

    use crate::{
        entities::{ArtistEntity, ArtistRefEntity, NewArtistRefEntity, UpdateArtistRefEntity}, macros
    };

    macros::resource::find_one!(artist_ref, ArtistRefEntity);
    macros::resource::find_all!(artist_ref, ArtistRefEntity);
    macros::resource::create!(artist_ref, ArtistRefEntity, NewArtistRefEntity);
    macros::resource::update!(artist_ref, ArtistRefEntity, UpdateArtistRefEntity);
    macros::resource::delete!(artist_ref);

    // custom operations

    pub fn get_artist_refs_by_artist_entity(
        conn: &mut SqliteConnection,
        artist: &ArtistEntity,
    ) -> Vec<ArtistRefEntity> {
        ArtistRefEntity::belonging_to(artist)
            .load::<ArtistRefEntity>(conn)
            .unwrap_or_default()
    }
}
