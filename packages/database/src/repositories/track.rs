use diesel::prelude::*;
use shared::types::SoundomeResult;

use crate::{
    entities::{AlbumEntity, ArtistEntity, ArtistTrackEntity, NewTrackEntity, TrackEntity, TrackRefEntity, UpdateTrackEntity}, macros, schema,
};

pub trait TrackRepository: Send + Sync {
    fn get_by_id(conn: &mut SqliteConnection, id: i32) -> SoundomeResult<shared::models::Track>;
    fn create(
        conn: &mut SqliteConnection,
        new_track: &shared::models::Track,
    ) -> SoundomeResult<shared::models::Track>;
}

pub struct DieselTrackRepository {}

impl TrackRepository for DieselTrackRepository {

    fn get_by_id(conn: &mut SqliteConnection, id: i32) -> SoundomeResult<shared::models::Track> {
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

    fn create(
        conn: &mut SqliteConnection,
        new_track: &shared::models::Track,
    ) -> SoundomeResult<shared::models::Track> {
        let new_track_entity = NewTrackEntity::convert_from_domain(new_track, None);
        let track = diesel::insert_into(schema::track::table)
            .values(&new_track_entity)
            .execute(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to create resource: {}",
                    err
                ))
            })?;

        Ok(track)
    }
}















// basic CRUD operations

macros::resource::find_one!(track, TrackEntity);
macros::resource::find_all!(track, TrackEntity);
macros::resource::create!(track, TrackEntity, NewTrackEntity);
macros::resource::update!(track, TrackEntity, UpdateTrackEntity);
macros::resource::delete!(track);

// associations

macros::association::many_to_many::get_all_associations!(
    get_artists,
    TrackEntity,
    artist,
    ArtistEntity,
    artist_tracks,
    ArtistTrackEntity,
    artist_id,
);

macros::association::many_to_many::create_association!(
    create_artist,
    TrackEntity,
    ArtistEntity,
    artist_tracks,
    ArtistTrackEntity,
    track_id,
    artist_id,
);

macros::association::many_to_many::delete_association!(
    delete_artist,
    TrackEntity,
    ArtistEntity,
    artist_tracks,
    track_id,
    artist_id,
);

pub fn get_album(conn: &mut SqliteConnection, track: &TrackEntity) -> Option<AlbumEntity> {

    match track.album_id {
        Some(album_id) => super::album::find_one(conn, album_id).ok(),
        None => None,
    }
}

// ================================================================================================
// Track Ref
// ================================================================================================

pub mod track_ref {
    use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

    use crate::{
        entities::{NewTrackRefEntity, TrackEntity, TrackRefEntity, UpdateTrackRefEntity}, macros
    };

    macros::resource::find_one!(track_ref, TrackRefEntity);
    macros::resource::find_all!(track_ref, TrackRefEntity);
    macros::resource::create!(track_ref, TrackRefEntity, NewTrackRefEntity);
    macros::resource::update!(track_ref, TrackRefEntity, UpdateTrackRefEntity);
    macros::resource::delete!(track_ref);

    // custom operations

    pub fn get_track_refs_by_track_entity(
        conn: &mut SqliteConnection,
        track: &TrackEntity,
    ) -> Vec<TrackRefEntity> {
        TrackRefEntity::belonging_to(track)
            .load::<TrackRefEntity>(conn)
            .unwrap_or_default()
    }
}
