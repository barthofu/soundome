use rsoundcloud::http::StatusCode;
use shared::{
    errors::Error,
    models::{
        album::{Album, AlbumType},
        artist::Artist,
        playlist::PlaylistTrack,
        track::{Track, TrackSource},
    },
};

/// Converts an rspotify ClientError into a shared Error.
pub fn convert_error(err: rsoundcloud::ClientError) -> Error {
    match err {
        rsoundcloud::ClientError::Http(ref http_error) => match http_error.as_ref() {
            rsoundcloud::http::HttpError::Client(err) => Error::Custom(err.to_string()),
            rsoundcloud::http::HttpError::StatusCode(response) => match response.status() {
                StatusCode::NOT_FOUND => Error::NotFound("Resource not found".to_string()),
                _ => Error::Http(response.status().to_string(), "".to_string()),
            },
        },
        // rustypipe::error::Error::Extraction(e) => Error::Custom(format!("Extraction error from Soundcloud: {}", e.to_string())),
        _ => Error::Unknown,
    }
}

/// Converts a Soundcloud artist to a shared Artist.
pub fn convert_artist(user: &rsoundcloud::models::user::User) -> Artist {
    Artist {
        name: user.user.full_name.clone(),
        url: Some(user.user.permalink_url.clone()),
        icon: Some(user.user.avatar_url.clone()),
    }
}

/// Converts a Soundcloud basic artist to a shared Artist.
pub fn convert_basic_artist(basic_user: &rsoundcloud::models::user::BasicUser) -> Artist {
    Artist {
        name: basic_user.full_name.clone(),
        url: Some(basic_user.permalink_url.clone()),
        icon: Some(basic_user.avatar_url.clone()),
    }
}

/// Converts a Soundcloud album to a shared Album.
pub fn convert_album(album_playlist: &rsoundcloud::models::playlist::AlbumPlaylist) -> Album {
    let user = &album_playlist.user;
    let album = &album_playlist.album_playlist;
    Album {
        title: album.title.clone(),
        artists: vec![convert_artist(&user)],
        album_type: AlbumType::Unknown,
        url: Some(album.permalink_url.clone()),
        cover: album.artwork_url.clone(),
        date: album.release_date.clone(),
    }
}

/// Converts a Soundcloud basic album to a shared Album.
pub fn convert_basic_album(
    basic_album_playlist: &rsoundcloud::models::playlist::BasicAlbumPlaylist,
) -> Album {
    let user = &basic_album_playlist.user;
    let album = &basic_album_playlist.album_playlist;
    Album {
        title: album.title.clone(),
        artists: vec![convert_basic_artist(&user)],
        album_type: AlbumType::Unknown,
        url: Some(album.permalink_url.clone()),
        cover: album.artwork_url.clone(),
        date: album.release_date.clone(),
    }
}

/// Converts a Soundcloud track to a shared Track.
pub fn convert_track(
    track: rsoundcloud::models::track::Track,
    album: Option<rsoundcloud::models::playlist::BasicAlbumPlaylist>,
) -> Track {
    let user = &track.user;
    let track = &track.track;
    Track {
        title: track.title.clone(),
        artists: vec![convert_basic_artist(&user)],
        album: album.as_ref().map(|a| convert_basic_album(&a)),
        genre: track.genre.clone(), // TODO: check if this is correct instead of tag_list
        duration: Some(track.duration.clone()),
        file_path: None,
        source: Some(TrackSource::SoundCloud),
        source_url: Some(track.permalink_url.clone()),
        source_id: Some(track.id.to_string()),
        provider: None,
        provider_url: None,
        provider_id: None,
        track_number: album.as_ref().and_then(|a| {
            a.album_playlist
                .tracks
                .iter()
                .position(|t| match t {
                    rsoundcloud::models::playlist::TrackType::Basic(b) => b.track.id == track.id,
                    rsoundcloud::models::playlist::TrackType::Mini(m) => m.id == track.id,
                })
                .map(|pos| (pos + 1) as i32)
        }),
        disc_number: None,
        label: track.label_name.clone(),
        date: track.release_date.clone(),
        cover: track.artwork_url.clone(),
    }
}

/// Converts a Soundcloud track to a shared PlaylistTrack.
pub fn convert_playlist_item(
    item: rsoundcloud::models::track::Track,
    pos: u32,
) -> Option<PlaylistTrack> {
    Some(PlaylistTrack {
        track: convert_track(item, None),
        added_at: None,
        position: Some(pos),
    })
}

// =======================================================================
// Processes
// =======================================================================
