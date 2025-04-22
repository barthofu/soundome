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
