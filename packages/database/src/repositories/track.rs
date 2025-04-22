use diesel::prelude::*;

use crate::{
    entities::{AlbumEntity, ArtistEntity, ArtistTrackEntity, NewTrackEntity, TrackEntity, UpdateTrackEntity}, macros,
};

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
// Track Source
// ================================================================================================

pub mod track_source {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

    use crate::{
        entities::{TrackSourceEntity, NewTrackSourceEntity, UpdateTrackSourceEntity}, macros
    };

    macros::resource::find_one!(track_source, TrackSourceEntity);
    macros::resource::find_all!(track_source, TrackSourceEntity);
    macros::resource::create!(track_source, TrackSourceEntity, NewTrackSourceEntity);
    macros::resource::update!(track_source, TrackSourceEntity, UpdateTrackSourceEntity);
    macros::resource::delete!(track_source);
}
