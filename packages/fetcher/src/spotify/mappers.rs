use rspotify::model::{
    FullAlbum, FullArtist, FullTrack, PlayableItem, PlaylistItem, SimplifiedAlbum,
    SimplifiedArtist, SimplifiedTrack,
};
use shared::{
    errors::Error,
    models::{Album, AlbumType, Artist, PlaylistTrack, Reference, ReferenceType, Track},
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
        id: None,
        name: artist.name.clone(),
        icon: None,
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: shared::models::Platform::Spotify,
            external_id: artist.id.as_ref().map(|id| id.to_string()),
            external_url: artist.external_urls.get("spotify").cloned(),
        }],
    }
}

/// Converts a full Spotify artist to a shared Artist.
pub fn convert_full_artist(artist: &FullArtist) -> Artist {
    Artist {
        id: None,
        name: artist.name.clone(),
        icon: artist.images.get(0).map(|image| image.url.clone()),
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: shared::models::Platform::Spotify,
            external_id: Some(artist.id.to_string()),
            external_url: artist.external_urls.get("spotify").cloned(),
        }],
    }
}

/// Converts a simplified Spotify album to a shared Album.
pub fn convert_simplified_album(album: &SimplifiedAlbum) -> Album {
    Album {
        id: None,
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
        cover: album.images.get(0).map(|image| image.url.clone()),
        date: album.release_date.clone(),
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: shared::models::Platform::Spotify,
            external_id: album.id.as_ref().map(|id| id.to_string()),
            external_url: album.external_urls.get("spotify").cloned(),
        }],
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
        id: None,
        title: album.name.clone(),
        artists: album.artists.iter().map(convert_artist).collect(),
        album_type: convert_album_type(&album.album_type),
        cover: album.images.get(0).map(|image| image.url.clone()),
        date: Some(album.release_date.clone()),
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: shared::models::Platform::Spotify,
            external_id: Some(album.id.to_string()),
            external_url: album.external_urls.get("spotify").cloned(),
        }],
    }
}

/// Converts a full Spotify track to a shared Track.
pub fn convert_track(track: &FullTrack) -> Track {
    let album = &track.album;
    let artists = track.artists.iter().map(convert_artist).collect();

    Track {
        id: None,
        needs_validation: false,
        validation_reason: None,
        title: track.name.clone(),
        artists,
        album: Some(convert_simplified_album(album)),
        genre: None, // TODO: get genre from artist
        duration: Some(track.duration.num_seconds() as i32),
        file_path: None,
        track_number: Some(track.track_number as i32),
        disc_number: Some(track.disc_number as i32),
        label: None,
        date: album.release_date.clone(),
        cover: album.images.get(0).map(|image| image.url.clone()),
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Source,
            platform: shared::models::Platform::Spotify,
            external_id: track.id.as_ref().map(|id| id.to_string()),
            external_url: track.external_urls.get("spotify").cloned(),
        }],
    }
}

/// Converts a Spotify playlist item to a shared PlaylistTrack.
pub fn convert_playlist_item(item: &PlaylistItem, pos: u32) -> Option<PlaylistTrack> {
    item.track.as_ref().and_then(|t| match t {
        PlayableItem::Track(track) => Some(PlaylistTrack {
            id: None,
            track: convert_track(track),
            added_at: item.added_at.clone(),
            position: Some(pos),
        }),
        PlayableItem::Episode(_) => None,
        PlayableItem::Unknown(_) => None,
    })
}

/// Converts a simplified Spotify track (from album listing) to a shared Track.
/// Requires the parent album to be passed separately for context.
pub fn convert_simplified_track(track: &SimplifiedTrack, album: &SimplifiedAlbum) -> Track {
    let artists = track.artists.iter().map(convert_artist).collect();

    Track {
        id: None,
        needs_validation: false,
        validation_reason: None,
        title: track.name.clone(),
        artists,
        album: Some(convert_simplified_album(album)),
        genre: None,
        duration: Some(track.duration.num_seconds() as i32),
        file_path: None,
        track_number: Some(track.track_number as i32),
        disc_number: Some(track.disc_number as i32),
        label: None,
        date: album.release_date.clone(),
        cover: album.images.get(0).map(|image| image.url.clone()),
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Source,
            platform: shared::models::Platform::Spotify,
            external_id: track.id.as_ref().map(|id| id.to_string()),
            external_url: track.external_urls.get("spotify").cloned(),
        }],
    }
}

// =========================================
// Utils
// =========================================

pub fn track_name_fixup(_track: &FullTrack) -> String {
    String::from("")
}
