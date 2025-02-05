use rspotify::model::{FullTrack, SimplifiedArtist, FullArtist, SimplifiedAlbum, FullAlbum, PlaylistItem, PlayableItem};

use shared::{models::{track::Track, artist::Artist, album::Album, playlist::PlaylistTrack}, errors::Error};

pub fn convert_error(err: rspotify::ClientError) -> Error {
    match err {
        rspotify::ClientError::ParseJson(e) => Error::Json(e),
        rspotify::ClientError::Http(_) => Error::Network,
        _ => Error::Other,
    }
}

pub fn convert_artist(artist: &SimplifiedArtist) -> Artist {
    Artist {
        name: artist.name.clone(),
        url: artist.external_urls.get("spotify").cloned(),
        icon: None,
    }
}

pub fn convert_full_artist(artist: &FullArtist) -> Artist {
    Artist {
        name: artist.name.clone(),
        url: artist.external_urls.get("spotify").cloned(),
        icon: artist.images.get(0).map(|image| image.url.clone()),
    }
}

pub fn convert_simplified_album(album: &SimplifiedAlbum) -> Album {
    Album {
        title: album.name.clone(),
        url: album.external_urls.get("spotify").cloned(),
        artists: album.artists.iter().map(convert_artist).collect(),
        cover: album.images.get(0).map(|image| image.url.clone()),
        date: album.release_date.clone(),
    }
}

pub fn convert_full_album(album: &FullAlbum) -> Album {
    Album {
        title: album.name.clone(),
        url: album.external_urls.get("spotify").cloned(),
        artists: album.artists.iter().map(convert_artist).collect(),
        cover: album.images.get(0).map(|image| image.url.clone()),
        date: Some(album.release_date.clone()),
    }
}

pub fn convert_track(track: &FullTrack) -> Track {
    let album = &track.album;
    let artists = track.artists.iter().map(convert_artist).collect();

    Track {
        title: track.name.clone(),
        artists,
        genre: None, // TODO: get genre from artist
        duration: Some((track.duration.num_seconds() & 0xffffffff) as i32),
        url: track.external_urls.get("spotify").cloned(),
        track_number: Some(track.track_number as i32),
        disc_number: Some(track.disc_number as i32),
        label: None,
        date: album.release_date.clone(),
        cover: album.images.get(0).map(|image| image.url.clone()),
        album: Some(convert_simplified_album(album)),
    }
}

pub fn convert_playlist_item(item: &PlaylistItem) -> PlaylistTrack {
    PlaylistTrack {
        track: item.track.as_ref().and_then(|t| match t {
            PlayableItem::Track(track) => Some(convert_track(track)),
            PlayableItem::Episode(_) => None,
        }),
        added_at: item.added_at.clone(),
        position: None,
    }
}
