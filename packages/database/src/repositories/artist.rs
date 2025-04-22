use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

use crate::{
    macros,
    entities::{
        ArtistEntity, ArtistTrackEntity, NewArtistEntity, UpdateArtistEntity, TrackEntity,
    },
};

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
// Artist Source
// ================================================================================================

pub mod artist_source {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

    use crate::{
        entities::{ArtistSourceEntity, NewArtistSourceEntity, UpdateArtistSourceEntity}, macros
    };

    macros::resource::find_one!(artist_source, ArtistSourceEntity);
    macros::resource::find_all!(artist_source, ArtistSourceEntity);
    macros::resource::create!(artist_source, ArtistSourceEntity, NewArtistSourceEntity);
    macros::resource::update!(artist_source, ArtistSourceEntity, UpdateArtistSourceEntity);
    macros::resource::delete!(artist_source);
}
