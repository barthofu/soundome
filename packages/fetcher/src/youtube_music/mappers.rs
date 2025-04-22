use rustypipe::model::{
    AlbumItem, AlbumType, ArtistId, ArtistItem, MusicAlbum, MusicArtist, TrackItem,
};
use shared::{
    errors::Error,
    models::{
        Album, Artist, Reference, Track
    },
};

/// Converts an rspotify ClientError into a shared Error.
pub fn convert_error(err: rustypipe::error::Error) -> Error {
    match err {
        rustypipe::error::Error::HttpStatus(status_code, _) => match status_code {
            404 => Error::NotFound("Resource not found".to_string()),
            _ => Error::Unknown,
        },
        rustypipe::error::Error::Extraction(e) => Error::Custom(format!(
            "Extraction error from Youtube Music: {}",
            e.to_string()
        )),
        _ => Error::Unknown,
    }
}

/// Converts a Youtube Music artist to a shared Artist.
pub fn convert_artist(artist: &MusicArtist) -> Artist {
    Artist {
        id: None,
        name: artist.name.clone(),
        icon: artist.header_image.first().map(|image| image.url.clone()),
        references: vec![Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Source,
            platform: shared::models::Platform::YoutubeMusic,
            external_id: Some(artist.id.to_string()),
            external_url: Some(format!("https://music.youtube.com/channel/{}", artist.id)),
        }],
    }
}

pub fn convert_artist_id(artist: &ArtistId) -> Artist {
    Artist {
        id: None,
        name: artist.name.clone(),
        icon: None,
        references: vec![Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Source,
            platform: shared::models::Platform::YoutubeMusic,
            external_id: artist.id.clone(),
            external_url: artist.id.clone().map(|id| format!("https://music.youtube.com/channel/{}", id)),
        }],
    }
}

pub fn convert_artist_item(artist: &ArtistItem) -> Artist {
    Artist {
        id: None,
        name: artist.name.clone(),
        icon: artist.avatar.first().map(|image| image.url.clone()),
        references: vec![Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Source,
            platform: shared::models::Platform::YoutubeMusic,
            external_id: Some(artist.id.to_string()),
            external_url: Some(format!("https://music.youtube.com/channel/{}", artist.id)),
        }],
    }
}

/// Converts a Youtube Music album to a shared Album.
pub fn convert_album(album: &MusicAlbum) -> Album {
    Album {
        id: None,
        title: album.name.clone(),
        artists: album.artists.iter().map(convert_artist_id).collect(),
        album_type: convert_album_type(album.album_type),
        cover: album.cover.first().map(|image| image.url.clone()),
        date: album.year.map(|year| year.to_string()),
        references: vec![Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Source,
            platform: shared::models::Platform::YoutubeMusic,
            external_id: Some(album.id.to_string()),
            external_url: Some(format!("https://music.youtube.com/channel/{}", album.id)),
        }],
    }
}

pub fn convert_album_item(album: &AlbumItem) -> Album {
    Album {
        id: None,
        title: album.name.clone(),
        artists: album.artists.iter().map(convert_artist_id).collect(),
        album_type: convert_album_type(album.album_type),
        cover: album.cover.first().map(|image| image.url.clone()),
        date: album.year.map(|year| year.to_string()),
        references: vec![Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Source,
            platform: shared::models::Platform::YoutubeMusic,
            external_id: Some(album.id.to_string()),
            external_url: Some(format!("https://music.youtube.com/channel/{}", album.id)),
        }],
    }
}

fn convert_album_type(album_type: AlbumType) -> shared::models::AlbumType {
    match album_type {
        AlbumType::Album => shared::models::AlbumType::Album,
        AlbumType::Single => shared::models::AlbumType::Single,
        AlbumType::Audiobook => shared::models::AlbumType::Unknown,
        _ => shared::models::AlbumType::Unknown,
    }
}

/// Converts a Youtube Music track to a shared Track.
pub fn convert_track(
    track: TrackItem,
    artists: Vec<MusicArtist>,
    album: Option<MusicAlbum>,
) -> Track {
    Track {
        id: None,
        title: track.name.clone(),
        artists: artists.iter().map(convert_artist).collect(),
        album: album.as_ref().map(|a| convert_album(a)),
        genre: None,
        duration: track.duration.map(|d| d as i32),
        file_path: None,
        track_number: track.track_nr.map(|n| n as i32),
        disc_number: None,
        label: None,
        date: album
            .as_ref()
            .and_then(|a| a.year.map(|year| year.to_string())),
        cover: album.and_then(|a| a.cover.first().map(|image| image.url.clone())),
        references: vec![Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Source,
            platform: shared::models::Platform::YoutubeMusic,
            external_id: Some(track.id.to_string()),
            external_url: Some(format!("https://music.youtube.com/watch?v={}", track.id)),
        }],
    }
}
