use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::types::SoundomeResult;

use crate::{
    entities::{
        ArtistEntity, ArtistRefEntity, ArtistTrackEntity, NewArtistEntity, NewArtistRefEntity, TrackEntity, UpdateArtistEntity
    }, macros, schema
};


pub trait ArtistRepository: Send + Sync {
    fn get_by_id(conn: &mut SqliteConnection, id: i32) -> SoundomeResult<shared::models::Artist>;
    fn create(
        conn: &mut SqliteConnection,
        new_track: &shared::models::Artist,
    ) -> SoundomeResult<shared::models::Artist>;
}

pub struct DieselArtistRepository {}

impl ArtistRepository for DieselArtistRepository {
    fn get_by_id(conn: &mut SqliteConnection, id: i32) -> SoundomeResult<shared::models::Artist> {
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

    fn create(
        conn: &mut SqliteConnection,
        new_artist: &shared::models::Artist,
    ) -> SoundomeResult<shared::models::Artist> {
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

        for reference in new_artist.references.clone() {
            let new_artist_ref = NewArtistRefEntity::convert_from_domain(&reference, inserted_artist.id);
            diesel::insert_into(schema::artist_ref::table)
                .values(&new_artist_ref)
                .execute(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to create resource: {}",
                        err
                    ))
                })?;
        }
        
        <DieselArtistRepository as ArtistRepository>::get_by_id(conn, inserted_artist.id)
    }
}







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
