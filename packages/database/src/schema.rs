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
    genre (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    playlist (id) {
        id -> Integer,
        name -> Text,
        source -> Text,
        source_url -> Nullable<Text>,
        cover -> Nullable<Text>,
        last_sync -> Nullable<Timestamp>,
    }
}

diesel::table! {
    playlist_tracks (id) {
        id -> Integer,
        track_id -> Integer,
        playlist_id -> Integer,
        position -> Integer,
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
        source_url -> Nullable<Text>,
        download_url -> Text,
        file_size -> Integer,
        file_path -> Text,
    }
}

diesel::table! {
    track_genres (id) {
        id -> Integer,
        track_id -> Integer,
        genre_id -> Integer,
    }
}

diesel::joinable!(album_tracks -> album (album_id));
diesel::joinable!(album_tracks -> track (track_id));
diesel::joinable!(artist_albums -> album (album_id));
diesel::joinable!(artist_albums -> artist (artist_id));
diesel::joinable!(artist_tracks -> artist (artist_id));
diesel::joinable!(artist_tracks -> track (track_id));
diesel::joinable!(playlist_tracks -> playlist (playlist_id));
diesel::joinable!(playlist_tracks -> track (track_id));
diesel::joinable!(track_genres -> genre (genre_id));
diesel::joinable!(track_genres -> track (track_id));

diesel::allow_tables_to_appear_in_same_query!(
    album,
    album_tracks,
    artist,
    artist_albums,
    artist_tracks,
    genre,
    playlist,
    playlist_tracks,
    track,
    track_genres,
);
