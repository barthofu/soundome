use std::sync::Arc;

pub mod track_service;
pub mod album_service;
pub mod artist_service;

pub struct ServiceLayer {
    pub track: Arc<track_service::TrackService>,
    pub album: Arc<album_service::AlbumService>,
    pub artist: Arc<artist_service::ArtistService>,
}

impl ServiceLayer {
    pub fn new(
        track_service: Arc<track_service::TrackService>,
        album_service: Arc<album_service::AlbumService>,
        artist_service: Arc<artist_service::ArtistService>,
    ) -> Self {
        Self {
            track: track_service,
            album: album_service,
            artist: artist_service,
        }
    }
}