use std::path::PathBuf;

use diesel::SqliteConnection;
use shared::{models::{Album, AlbumType, Track, TrackProvider, TrackSource}, types::SoundomeResult};

use crate::{entities::{AlbumEntity, ArtistEntity, NewAlbumEntity, NewArtistEntity, NewTrackEntity, TrackEntity}, repositories};

// ================================================================================================
// Track
// ================================================================================================

pub fn convert_track_entity_to_track(conn: &mut SqliteConnection, track_entity: TrackEntity) -> SoundomeResult<Track> {

    let album = repositories::track::get_album(conn, &track_entity)
        .map(|album_entity| convert_album_entity_to_track(conn, album_entity))
        .transpose()?;
    let artists = repositories::track::get_artists(conn, &track_entity)?
        .into_iter()
        .map(|artist_entity| convert_artist_entity_to_artist(artist_entity))
        .collect();

    Ok(Track {
        id: Some(track_entity.id),
        title: track_entity.title,
        duration: track_entity.duration,
        track_number: track_entity.track_number,
        disc_number: track_entity.disc_number,
        label: track_entity.label,
        date: track_entity.date,
        genre: track_entity.genre,
        cover: track_entity.cover,
        source: track_entity.source.map(TrackSource::from_string),
        source_url: track_entity.source_url,
        source_id: track_entity.source_id,
        provider: track_entity.provider.map(TrackProvider::from_string),
        provider_url: track_entity.provider_url,
        provider_id: track_entity.provider_id,
        file_path: track_entity.file_path.map(PathBuf::from),
        album,
        artists,
    })
}

pub fn convert_track_to_new_track_entity(track: &shared::models::Track, album_id: Option<i32>) -> NewTrackEntity {
    NewTrackEntity {
        title: track.title.clone(),
        duration: track.duration,
        track_number: track.track_number,
        disc_number: track.disc_number,
        album_id,
        label: track.label.clone(),
        date: track.date.clone(),
        genre: track.genre.clone(),
        cover: track.cover.clone(),
        source: track.source.clone().map(|s| s.as_ref().to_string()),
        source_url: track.source_url.clone(),
        source_id: track.source_id.clone(),
        provider: track.provider.clone().map(|p| p.as_ref().to_string()),
        provider_url: track.provider_url.clone(),
        provider_id: track.provider_id.clone(),
        file_path: track.file_path.clone().map(|p| p.to_string_lossy().to_string()),
    }
}

// ================================================================================================
// Album
// ================================================================================================


pub fn convert_album_entity_to_track(conn: &mut SqliteConnection, album_entity: AlbumEntity) -> SoundomeResult<Album> {

    let artists = repositories::album::get_artists(conn, &album_entity)?
        .into_iter()
        .map(|artist_entity| convert_artist_entity_to_artist(artist_entity))
        .collect();

    Ok(Album {
        id: Some(album_entity.id),
        title: album_entity.title,
        artists,
        album_type: AlbumType::from_str(&album_entity.album_type),
        url: album_entity.url,
        cover: album_entity.cover,
        date: album_entity.date,
    })
}

pub fn convert_album_to_new_album_entity(album: &shared::models::Album) -> NewAlbumEntity {
    NewAlbumEntity {
        title: album.title.clone(),
        album_type: album.album_type.as_ref().to_string(),
        url: album.url.clone(),
        cover: album.cover.clone(),
        date: album.date.clone(),
    }
}

// ================================================================================================
// Artist
// ================================================================================================

pub fn convert_artist_entity_to_artist(artist_entity: ArtistEntity) -> shared::models::Artist {
    shared::models::Artist {
        id: Some(artist_entity.id),
        name: artist_entity.name,
        url: artist_entity.url,
        icon: artist_entity.icon,
    }
}

pub fn convert_artist_to_new_artist_entity(artist: &shared::models::Artist) -> NewArtistEntity {
    NewArtistEntity {
        name: artist.name.clone(),
        url: artist.url.clone(),
        icon: artist.icon.clone(),
    }
}

