// @generated automatically by Diesel CLI.

diesel::table! {
    album (id) {
        id -> Integer,
        title -> Text,
        cover -> Nullable<Text>,
        year -> Nullable<Integer>,
    }
}

diesel::table! {
    album_tracks (id) {
        id -> Integer,
        track_id -> Integer,
        album_id -> Integer,
    }
}

diesel::table! {
    artist (id) {
        id -> Integer,
        name -> Text,
        icon -> Nullable<Text>,
    }
}

diesel::table! {
    artist_albums (id) {
        id -> Integer,
        album_id -> Integer,
        artist_id -> Integer,
    }
}

diesel::table! {
    artist_tracks (id) {
        id -> Integer,
        track_id -> Integer,
        artist_id -> Integer,
    }
}

diesel::table! {
    track (id) {
        id -> Integer,
        title -> Text,
        album -> Nullable<Text>,
        year -> Nullable<Integer>,
        cover -> Nullable<Text>,
        duration -> Nullable<Integer>,
        track_number -> Nullable<Integer>,
        disc_number -> Nullable<Integer>,
        comments -> Nullable<Text>,
        label -> Nullable<Text>,
        file_type -> Text,
        file_size -> Integer,
        file_path -> Text,
    }
}

diesel::joinable!(album_tracks -> album (album_id));
diesel::joinable!(album_tracks -> track (track_id));
diesel::joinable!(artist_albums -> album (album_id));
diesel::joinable!(artist_albums -> artist (artist_id));
diesel::joinable!(artist_tracks -> artist (artist_id));
diesel::joinable!(artist_tracks -> track (track_id));

diesel::allow_tables_to_appear_in_same_query!(
    album,
    album_tracks,
    artist,
    artist_albums,
    artist_tracks,
    track,
);
