// @generated automatically by Diesel CLI.

diesel::table! {
    album (id) {
        id -> Integer,
        title -> Text,
        album_type -> Text,
        cover -> Nullable<Text>,
        date -> Nullable<Text>,
    }
}

diesel::table! {
    album_ref (id) {
        id -> Integer,
        album_id -> Integer,
        #[sql_name = "type"]
        type_ -> Text,
        platform -> Text,
        external_id -> Nullable<Text>,
        external_url -> Nullable<Text>,
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
    artist_albums (album_id, artist_id) {
        album_id -> Integer,
        artist_id -> Integer,
    }
}

diesel::table! {
    artist_ref (id) {
        id -> Integer,
        artist_id -> Integer,
        #[sql_name = "type"]
        type_ -> Text,
        platform -> Text,
        external_id -> Nullable<Text>,
        external_url -> Nullable<Text>,
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
    sync_schedule (id) {
        id -> Integer,
        playlist_url -> Text,
        label -> Nullable<Text>,
        interval_seconds -> Integer,
        enabled -> Integer,
        last_run -> Nullable<Timestamp>,
        next_run -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    task (id) {
        id -> Integer,
        task_type -> Text,
        status -> Text,
        payload -> Text,
        label -> Nullable<Text>,
        progress -> Integer,
        total -> Nullable<Integer>,
        error -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        needs_validation -> Bool,
        validation_reason -> Nullable<Text>,
    }
}

diesel::table! {
    track_genres (track_id, genre_id) {
        track_id -> Integer,
        genre_id -> Integer,
    }
}

diesel::table! {
    track_ref (id) {
        id -> Integer,
        track_id -> Integer,
        #[sql_name = "type"]
        type_ -> Text,
        platform -> Text,
        external_id -> Nullable<Text>,
        external_url -> Nullable<Text>,
    }
}

diesel::joinable!(album_ref -> album (album_id));
diesel::joinable!(artist_albums -> album (album_id));
diesel::joinable!(artist_albums -> artist (artist_id));
diesel::joinable!(artist_ref -> artist (artist_id));
diesel::joinable!(artist_tracks -> artist (artist_id));
diesel::joinable!(artist_tracks -> track (track_id));
diesel::joinable!(playlist_tracks -> playlist (playlist_id));
diesel::joinable!(playlist_tracks -> track (track_id));
diesel::joinable!(track -> album (album_id));
diesel::joinable!(track_genres -> genre (genre_id));
diesel::joinable!(track_genres -> track (track_id));
diesel::joinable!(track_ref -> track (track_id));

diesel::allow_tables_to_appear_in_same_query!(
    album,
    album_ref,
    artist,
    artist_albums,
    artist_ref,
    artist_tracks,
    genre,
    playlist,
    playlist_tracks,
    sync_schedule,
    task,
    track,
    track_genres,
    track_ref,
);
