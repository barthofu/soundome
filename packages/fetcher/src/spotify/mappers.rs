use rspotify::model::{FullTrack, SimplifiedArtist, FullArtist, SimplifiedAlbum, FullAlbum, PlaylistItem, PlayableItem};

use crate::{models::{track::Track, artist::Artist, album::Album, playlist::PlaylistTrack}, utils::errors::Error};

impl From<rspotify::ClientError> for Error {
    fn from(err: rspotify::ClientError) -> Self {
        match err {
            rspotify::ClientError::ParseJson(e) => Error::Json(e),
            rspotify::ClientError::Http(_) => Error::Network,
            _ => Error::Other,
        }
    }
}

impl From<FullTrack> for Track {
    fn from(track: FullTrack) -> Self {

        let album = track.album;
        let artists = track.artists.into_iter().map(|artist| artist.into()).collect();

        Self {
            title: track.name,
            artists: artists,
            genre: None, // TODO: get genre from artist
            duration: Some((track.duration.as_secs() & 0xffffffff) as i32),
            url: track.external_urls.get("spotify").cloned(),
            track_number: Some(track.track_number as i32),
            disc_number: Some(track.disc_number as i32),
            label: None,
            date: album.release_date.clone(),
            cover: album.images.get(0).map(|image| image.url.clone()),
            album: Some(<SimplifiedAlbum as Into<Album>>::into(album)),
        }
    }
}

impl From<SimplifiedArtist> for Artist {
    fn from(artist: SimplifiedArtist) -> Self {

        Self {
            name: artist.name,
            url: artist.external_urls.get("spotify").cloned(),
            icon: None,
        }
    }
}

impl From<FullArtist> for Artist {
    fn from(artist: FullArtist) -> Self {

        Self {
            name: artist.name,
            url: artist.external_urls.get("spotify").cloned(),
            icon: artist.images.get(0).map(|image| image.url.clone()),
        }
    }
}

impl From<SimplifiedAlbum> for Album {
    fn from(album: SimplifiedAlbum) -> Self {
        Self {
            title: album.name,
            url: album.external_urls.get("spotify").cloned(),
            artists: album.artists.into_iter().map(|artist| artist.into()).collect(),
            cover: album.images.get(0).map(|image| image.url.clone()),
            date: album.release_date
        }
    }
}

impl From<FullAlbum> for Album {
    fn from(album: FullAlbum) -> Self {
        Self {
            title: album.name,
            url: album.external_urls.get("spotify").cloned(),
            artists: album.artists.into_iter().map(|artist| artist.into()).collect(),
            cover: album.images.get(0).map(|image| image.url.clone()),
            date: Some(album.release_date)
        }
    }
}

impl From<PlaylistItem> for PlaylistTrack {

    fn from(item: PlaylistItem) -> Self {
        Self {
            track: match item.track {
                Some(item) => match item {
                    PlayableItem::Track(track) => Some(track.into()),
                    PlayableItem::Episode(_) => None,
                },
                None => None,
            },
            added_at: item.added_at,
            position: None,
        }
    }
}
