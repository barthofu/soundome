use rspotify::model::{
    FullAlbum, FullArtist, FullTrack, PlayableItem, PlaylistItem, SimplifiedAlbum, SimplifiedArtist,
};
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
pub fn convert_error(err: rspotify::ClientError) -> Error {
    match err {
        rspotify::ClientError::ParseJson(e) => Error::Json(e),
        rspotify::ClientError::Http(e) => Error::Network(e.to_string()),
        _ => Error::Unknown,
    }
}

/// Converts a simplified Spotify artist to a shared Artist.
pub fn convert_artist(artist: &SimplifiedArtist) -> Artist {
    Artist {
        name: artist.name.clone(),
        url: artist.external_urls.get("spotify").cloned(),
        icon: None,
    }
}

/// Converts a full Spotify artist to a shared Artist.
pub fn convert_full_artist(artist: &FullArtist) -> Artist {
    Artist {
        name: artist.name.clone(),
        url: artist.external_urls.get("spotify").cloned(),
        icon: artist.images.get(0).map(|image| image.url.clone()),
    }
}

/// Converts a simplified Spotify album to a shared Album.
pub fn convert_simplified_album(album: &SimplifiedAlbum) -> Album {
    Album {
        title: album.name.clone(),
        artists: album.artists.iter().map(convert_artist).collect(),
        album_type: album
            .album_type
            .as_ref()
            .map(|album_type| match album_type {
                s if s == "album" => AlbumType::Album,
                s if s == "single" => AlbumType::Single,
                s if s == "compilation" => AlbumType::Compilation,
                _ => AlbumType::Unknown,
            })
            .unwrap_or(AlbumType::Unknown),
        url: album.external_urls.get("spotify").cloned(),
        cover: album.images.get(0).map(|image| image.url.clone()),
        date: album.release_date.clone(),
    }
}

fn convert_album_type(album_type: &rspotify::model::AlbumType) -> AlbumType {
    match album_type {
        rspotify::model::AlbumType::Album => AlbumType::Album,
        rspotify::model::AlbumType::Single => AlbumType::Single,
        rspotify::model::AlbumType::Compilation => AlbumType::Compilation,
        _ => AlbumType::Unknown,
    }
}

/// Converts a full Spotify album to a shared Album.
pub fn convert_full_album(album: &FullAlbum) -> Album {
    Album {
        title: album.name.clone(),
        artists: album.artists.iter().map(convert_artist).collect(),
        album_type: convert_album_type(&album.album_type),
        url: album.external_urls.get("spotify").cloned(),
        cover: album.images.get(0).map(|image| image.url.clone()),
        date: Some(album.release_date.clone()),
    }
}

/// Converts a full Spotify track to a shared Track.
pub fn convert_track(track: &FullTrack) -> Track {
    let album = &track.album;
    let artists = track.artists.iter().map(convert_artist).collect();

    Track {
        title: track.name.clone(),
        artists,
        album: Some(convert_simplified_album(album)),
        genre: None, // TODO: get genre from artist
        duration: Some(track.duration.num_seconds() as i32),
        file_path: None,
        source: Some(TrackSource::Spotify),
        source_url: track.external_urls.get("spotify").cloned(),
        provider: None,
        provider_url: None,
        track_number: Some(track.track_number as i32),
        disc_number: Some(track.disc_number as i32),
        label: None,
        date: album.release_date.clone(),
        cover: album.images.get(0).map(|image| image.url.clone()),
    }
}

/// Converts a Spotify playlist item to a shared PlaylistTrack.
pub fn convert_playlist_item(item: &PlaylistItem, pos: u32) -> Option<PlaylistTrack> {
    item.track.as_ref().and_then(|t| match t {
        PlayableItem::Track(track) => Some(PlaylistTrack {
            track: convert_track(track),
            added_at: item.added_at.clone(),
            position: Some(pos),
        }),
        PlayableItem::Episode(_) => None,
    })
}

// =========================================
// Utils
// =========================================

pub fn track_name_fixup(_track: &FullTrack) -> String {
    String::from("")
}
