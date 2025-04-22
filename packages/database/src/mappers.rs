use std::path::PathBuf;

use diesel::SqliteConnection;
use shared::{models::{Album, AlbumType, Track}, types::SoundomeResult};

use crate::{entities::{AlbumEntity, AlbumRefEntity, ArtistEntity, ArtistRefEntity, NewAlbumEntity, NewAlbumRefEntity, NewArtistEntity, NewArtistRefEntity, NewTrackEntity, NewTrackRefEntity, TrackEntity, TrackRefEntity}, repositories};

// ================================================================================================
// Track
// ================================================================================================

pub fn convert_track_entity_to_track(conn: &mut SqliteConnection, track_entity: TrackEntity) -> SoundomeResult<Track> {

    let album = repositories::track::get_album(conn, &track_entity)
        .map(|album_entity| convert_album_entity_to_album(conn, album_entity))
        .transpose()?;
    let artists = repositories::track::get_artists(conn, &track_entity)?
        .into_iter()
        .map(|artist_entity| convert_artist_entity_to_artist(conn, artist_entity))
        .collect();
    let references = repositories::track::track_ref::get_track_refs_by_track_entity(conn, &track_entity)
        .iter()
        .map(convert_track_ref_entity_to_ref)
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
        file_path: track_entity.file_path.map(PathBuf::from),
        album,
        artists,
        references,
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
        file_path: track.file_path.clone().map(|p| p.to_string_lossy().to_string()),
    }
}

// ================================================================================================
// Album
// ================================================================================================


pub fn convert_album_entity_to_album(conn: &mut SqliteConnection, album_entity: AlbumEntity) -> SoundomeResult<Album> {

    let artists = repositories::album::get_artists(conn, &album_entity)?
        .into_iter()
        .map(|artist_entity| convert_artist_entity_to_artist(conn, artist_entity))
        .collect();

    let references = repositories::album::album_ref::get_album_refs_by_album_entity(conn, &album_entity)
        .iter()
        .map(convert_album_ref_entity_to_ref)
        .collect();

    Ok(Album {
        id: Some(album_entity.id),
        title: album_entity.title,
        artists,
        album_type: AlbumType::from_str(&album_entity.album_type),
        cover: album_entity.cover,
        date: album_entity.date,
        references,
    })
}

pub fn convert_album_to_new_album_entity(album: &shared::models::Album) -> NewAlbumEntity {
    NewAlbumEntity {
        title: album.title.clone(),
        album_type: album.album_type.as_ref().to_string(),
        cover: album.cover.clone(),
        date: album.date.clone(),
        // url: album.url.clone(),
    }
}

// ================================================================================================
// Artist
// ================================================================================================

pub fn convert_artist_entity_to_artist(conn: &mut SqliteConnection, artist_entity: ArtistEntity) -> shared::models::Artist {

    let references = repositories::artist::artist_ref::get_artist_refs_by_artist_entity(conn, &artist_entity)
        .iter()
        .map(convert_artist_ref_entity_to_ref)
        .collect();

    shared::models::Artist {
        id: Some(artist_entity.id),
        name: artist_entity.name,
        icon: artist_entity.icon,
        references,
    }
}

pub fn convert_artist_to_new_artist_entity(artist: &shared::models::Artist) -> NewArtistEntity {
    NewArtistEntity {
        name: artist.name.clone(),
        icon: artist.icon.clone(),
        // url: artist.url.clone(),
    }
}

// ================================================================================================
// References
// ================================================================================================

pub fn convert_album_ref_entity_to_ref(album_ref_entity: &AlbumRefEntity) -> shared::models::Reference {
    shared::models::Reference {
        id: Some(album_ref_entity.id),
        ref_type: shared::models::ReferenceType::from_str(&album_ref_entity.ref_type),
        platform: shared::models::Platform::from_str(&album_ref_entity.platform),
        external_id: album_ref_entity.external_id.clone(),
        external_url: album_ref_entity.external_url.clone(),
    }
}

pub fn convert_album_ref_to_new_album_ref_entity(album_ref: &shared::models::Reference, album_id: i32) -> NewAlbumRefEntity {
    NewAlbumRefEntity {
        album_id,
        ref_type: album_ref.ref_type.as_ref().to_string(),
        platform: album_ref.platform.as_ref().to_string(),
        external_id: album_ref.external_id.clone(),
        external_url: album_ref.external_url.clone(),
    }
}

pub fn convert_track_ref_entity_to_ref(track_ref_entity: &TrackRefEntity) -> shared::models::Reference {
    shared::models::Reference {
        id: Some(track_ref_entity.id),
        ref_type: shared::models::ReferenceType::from_str(&track_ref_entity.ref_type),
        platform: shared::models::Platform::from_str(&track_ref_entity.platform),
        external_id: track_ref_entity.external_id.clone(),
        external_url: track_ref_entity.external_url.clone(),
    }
}

pub fn convert_track_ref_to_new_track_ref_entity(track_ref: &shared::models::Reference, track_id: i32) -> NewTrackRefEntity {
    NewTrackRefEntity {
        track_id,
        ref_type: track_ref.ref_type.as_ref().to_string(),
        platform: track_ref.platform.as_ref().to_string(),
        external_id: track_ref.external_id.clone(),
        external_url: track_ref.external_url.clone(),
    }
}

pub fn convert_artist_ref_entity_to_ref(artist_ref_entity: &ArtistRefEntity) -> shared::models::Reference {
    shared::models::Reference {
        id: Some(artist_ref_entity.id),
        ref_type: shared::models::ReferenceType::from_str(&artist_ref_entity.ref_type),
        platform: shared::models::Platform::from_str(&artist_ref_entity.platform),
        external_id: artist_ref_entity.external_id.clone(),
        external_url: artist_ref_entity.external_url.clone(),
    }
}

pub fn convert_artist_ref_to_new_artist_ref_entity(artist_ref: &shared::models::Reference, artist_id: i32) -> NewArtistRefEntity {
    NewArtistRefEntity {
        artist_id,
        ref_type: artist_ref.ref_type.as_ref().to_string(),
        platform: artist_ref.platform.as_ref().to_string(),
        external_id: artist_ref.external_id.clone(),
        external_url: artist_ref.external_url.clone(),
    }
}
