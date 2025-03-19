use diesel::prelude::*;

use crate::{
    macros,
    models::{
        artist::{ArtistEntity, ArtistTrack, NewArtist, NewArtistTrack, UpdateArtist},
        track::TrackEntity,
    },
};

// basic CRUD operations

macros::resource::find_one!(artist, ArtistEntity);
macros::resource::find_all!(artist, ArtistEntity);
macros::resource::create!(artist, ArtistEntity, NewArtist);
macros::resource::update!(artist, ArtistEntity, UpdateArtist);
macros::resource::delete!(artist);

// associations

macros::association::get_all_associations!(
    get_tracks,
    ArtistEntity,
    track,
    TrackEntity,
    artist_tracks,
    ArtistTrack,
    track_id,
);

macros::association::create_association!(
    create_track,
    ArtistEntity,
    TrackEntity,
    artist_tracks,
    ArtistTrack,
    NewArtistTrack,
    artist_id,
    track_id,
);

macros::association::delete_association!(
    delete_track,
    ArtistEntity,
    TrackEntity,
    artist_tracks,
    artist_id,
    track_id,
);

// custom operations

pub fn has_track(conn: &mut SqliteConnection, artist: &ArtistEntity, track: &TrackEntity) -> bool {
    ArtistTrack::belonging_to(artist)
        .select(crate::schema::artist_tracks::track_id)
        .filter(crate::schema::artist_tracks::track_id.eq(track.id))
        .first::<i32>(conn)
        .is_ok()
}
