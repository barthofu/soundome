use diesel::prelude::*;

use crate::{
    models::{
        artist::{Artist, ArtistTrack, NewArtist, NewArtistTrack, UpdateArtist},
        track::Track,
    },
    utils::macros,
};

// basic CRUD operations

macros::crud_database::resource::find_one!(artist, Artist);
macros::crud_database::resource::find_all!(artist, Artist);
macros::crud_database::resource::create!(artist, Artist, NewArtist);
macros::crud_database::resource::update!(artist, Artist, UpdateArtist);
macros::crud_database::resource::delete!(artist);

// associations

macros::crud_database::association::get_all_associations!(
    get_tracks,
    Artist,
    track,
    Track,
    artist_tracks,
    ArtistTrack,
    track_id,
);

macros::crud_database::association::create_association!(
    create_track,
    Artist,
    Track,
    artist_tracks,
    ArtistTrack,
    NewArtistTrack,
    artist_id,
    track_id,
);

macros::crud_database::association::delete_association!(
    delete_track,
    Artist,
    Track,
    artist_tracks,
    artist_id,
    track_id,
);

pub fn has_track(conn: &mut SqliteConnection, artist: &Artist, track: &Track) -> bool {
    ArtistTrack::belonging_to(artist)
        .select(crate::schema::artist_tracks::track_id)
        .filter(crate::schema::artist_tracks::track_id.eq(track.id))
        .first::<i32>(conn)
        .is_ok()
}

// custom operations
