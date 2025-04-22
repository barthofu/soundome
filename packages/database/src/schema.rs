// @generated automatically by Diesel CLI.

diesel::table! {
    album (id) {
        id -> Integer,
        title -> Text,
        album_type -> Text,
        cover -> Nullable<Text>,
        date -> Nullable<Text>,
        url -> Nullable<Text>,
    }
}

diesel::table! {
    album_source (id) {
        id -> Integer,
        album_id -> Integer,
        external_id -> Text,
        platform -> Text,
    }
}

diesel::table! {
    artist (id) {
        id -> Integer,
        name -> Text,
        url -> Nullable<Text>,
        icon -> Nullable<Text>,
    }
}

diesel::table! {
    artist_albums (album_id, artist_id) {
        album_id -> Integer,
        artist_id -> Integer,
    }
}

diesel::table! {
    artist_source (id) {
        id -> Integer,
        artist_id -> Integer,
        external_id -> Text,
        platform -> Text,
    }
}

diesel::table! {
    artist_tracks (track_id, artist_id) {
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
    playlist_tracks (track_id, playlist_id) {
        track_id -> Integer,
        playlist_id -> Integer,
        position -> Nullable<Integer>,
    }
}

diesel::table! {
    track (id) {
        id -> Integer,
        title -> Text,
        duration -> Nullable<Integer>,
        album_id -> Nullable<Integer>,
        track_number -> Nullable<Integer>,
        disc_number -> Nullable<Integer>,
        label -> Nullable<Text>,
        date -> Nullable<Text>,
        genre -> Nullable<Text>,
        cover -> Nullable<Text>,
        file_path -> Nullable<Text>,
        source -> Nullable<Text>,
        source_url -> Nullable<Text>,
        source_id -> Nullable<Text>,
        provider -> Nullable<Text>,
        provider_url -> Nullable<Text>,
        provider_id -> Nullable<Text>,
    }
}

diesel::table! {
    track_genres (track_id, genre_id) {
        track_id -> Integer,
        genre_id -> Integer,
    }
}

diesel::table! {
    track_source (id) {
        id -> Integer,
        track_id -> Integer,
        external_id -> Text,
        platform -> Text,
    }
}

diesel::joinable!(album_source -> album (album_id));
diesel::joinable!(artist_albums -> album (album_id));
diesel::joinable!(artist_albums -> artist (artist_id));
diesel::joinable!(artist_source -> artist (artist_id));
diesel::joinable!(artist_tracks -> artist (artist_id));
diesel::joinable!(artist_tracks -> track (track_id));
diesel::joinable!(playlist_tracks -> playlist (playlist_id));
diesel::joinable!(playlist_tracks -> track (track_id));
diesel::joinable!(track -> album (album_id));
diesel::joinable!(track_genres -> genre (genre_id));
diesel::joinable!(track_genres -> track (track_id));
diesel::joinable!(track_source -> track (track_id));

diesel::allow_tables_to_appear_in_same_query!(
    album,
    album_source,
    artist,
    artist_albums,
    artist_source,
    artist_tracks,
    genre,
    playlist,
    playlist_tracks,
    track,
    track_genres,
    track_source,
);
