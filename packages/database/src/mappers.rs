use std::path::PathBuf;

use shared::models::{Album, AlbumType};

use crate::entities::{AlbumEntity, AlbumRefEntity, ArtistEntity, ArtistRefEntity, NewAlbumEntity, NewAlbumRefEntity, NewArtistEntity, NewArtistRefEntity, NewTrackEntity, NewTrackRefEntity, TrackEntity, TrackRefEntity};

// ================================================================================================
// Track
// ================================================================================================

impl TrackEntity {

    pub fn convert_to_domain(track_entity: TrackEntity, album: Option<AlbumEntity>, artists: Vec<ArtistEntity>, references: Vec<TrackRefEntity>) -> shared::models::Track {
        shared::models::Track {
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
            album: album.map(|a| AlbumEntity::convert_to_domain(a, vec![], vec![])),
            artists: artists.into_iter().map(|a| ArtistEntity::convert_to_domain(a, vec![])).collect(),
            references: references.into_iter().map(TrackRefEntity::convert_to_domain).collect(),
        }
    }
}

impl NewTrackEntity {

    pub fn convert_from_domain(track: &shared::models::Track, album_id: Option<i32>) -> NewTrackEntity {
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
}

// ================================================================================================
// Album
// ================================================================================================

impl AlbumEntity {

    pub fn convert_to_domain(album_entity: AlbumEntity, artists: Vec<ArtistEntity>, references: Vec<AlbumRefEntity>) -> Album {
        Album {
            id: Some(album_entity.id),
            title: album_entity.title,
            artists: artists.into_iter().map(|a| ArtistEntity::convert_to_domain(a, vec![])).collect(),
            album_type: AlbumType::from_str(&album_entity.album_type),
            cover: album_entity.cover,
            date: album_entity.date,
            references: references.into_iter().map(AlbumRefEntity::convert_to_domain).collect(),
        }
    }
}

impl NewAlbumEntity {

    pub fn convert_from_domain(album: &shared::models::Album) -> NewAlbumEntity {
        NewAlbumEntity {
            title: album.title.clone(),
            album_type: album.album_type.as_ref().to_string(),
            cover: album.cover.clone(),
            date: album.date.clone(),
        }
    }
}

// ================================================================================================
// Artist
// ================================================================================================

impl ArtistEntity {

    pub fn convert_to_domain(artist_entity: ArtistEntity, references: Vec<ArtistRefEntity>) -> shared::models::Artist {
        shared::models::Artist {
            id: Some(artist_entity.id),
            name: artist_entity.name,
            icon: artist_entity.icon,
            references: references.into_iter().map(ArtistRefEntity::convert_to_domain).collect(),
        }
    }
}

impl NewArtistEntity {

    pub fn convert_from_domain(artist: &shared::models::Artist) -> NewArtistEntity {
        NewArtistEntity {
            name: artist.name.clone(),
            icon: artist.icon.clone(),
        }
    }
}

// ================================================================================================
// References
// ================================================================================================

impl TrackRefEntity {

    pub fn convert_to_domain(track_ref_entity: TrackRefEntity) -> shared::models::Reference {
        shared::models::Reference {
            id: Some(track_ref_entity.id),
            ref_type: shared::models::ReferenceType::from_str(&track_ref_entity.ref_type),
            platform: shared::models::Platform::from_str(&track_ref_entity.platform),
            external_id: track_ref_entity.external_id.clone(),
            external_url: track_ref_entity.external_url.clone(),
        }
    }
}

impl NewTrackRefEntity {

    pub fn convert_from_domain(track_ref: &shared::models::Reference, track_id: i32) -> NewTrackRefEntity {
        NewTrackRefEntity {
            track_id,
            ref_type: track_ref.ref_type.as_ref().to_string(),
            platform: track_ref.platform.as_ref().to_string(),
            external_id: track_ref.external_id.clone(),
            external_url: track_ref.external_url.clone(),
        }
    }
}

impl AlbumRefEntity {

    pub fn convert_to_domain(album_ref_entity: AlbumRefEntity) -> shared::models::Reference {
        shared::models::Reference {
            id: Some(album_ref_entity.id),
            ref_type: shared::models::ReferenceType::from_str(&album_ref_entity.ref_type),
            platform: shared::models::Platform::from_str(&album_ref_entity.platform),
            external_id: album_ref_entity.external_id.clone(),
            external_url: album_ref_entity.external_url.clone(),
        }
    }
}

impl NewAlbumRefEntity {

    pub fn convert_from_domain(album_ref: &shared::models::Reference, album_id: i32) -> NewAlbumRefEntity {
        NewAlbumRefEntity {
            album_id,
            ref_type: album_ref.ref_type.as_ref().to_string(),
            platform: album_ref.platform.as_ref().to_string(),
            external_id: album_ref.external_id.clone(),
            external_url: album_ref.external_url.clone(),
        }
    }
}

impl ArtistRefEntity {

    pub fn convert_to_domain(artist_ref_entity: ArtistRefEntity) -> shared::models::Reference {
        shared::models::Reference {
            id: Some(artist_ref_entity.id),
            ref_type: shared::models::ReferenceType::from_str(&artist_ref_entity.ref_type),
            platform: shared::models::Platform::from_str(&artist_ref_entity.platform),
            external_id: artist_ref_entity.external_id.clone(),
            external_url: artist_ref_entity.external_url.clone(),
        }
    }
}

impl NewArtistRefEntity {

    pub fn convert_from_domain(artist_ref: &shared::models::Reference, artist_id: i32) -> NewArtistRefEntity {
        NewArtistRefEntity {
            artist_id,
            ref_type: artist_ref.ref_type.as_ref().to_string(),
            platform: artist_ref.platform.as_ref().to_string(),
            external_id: artist_ref.external_id.clone(),
            external_url: artist_ref.external_url.clone(),
        }
    }
}
