use std::sync::Arc;

mod resources;
pub use resources::*;

use crate::ports::repositories;

pub mod download_service;

// =================================================================================
// Service Layer
// =================================================================================

pub struct ServiceLayer {
    pub track_service: Arc<track_service::TrackService>,
    pub album_service: Arc<album_service::AlbumService>,
    pub artist_service: Arc<artist_service::ArtistService>,
    pub playlist_service: Arc<playlist_service::PlaylistService>,
    pub sync_schedule_service: Arc<sync_schedule_service::SyncScheduleService>,
    pub task_service: Arc<task_service::TaskService>,

    pub download_service: Arc<download_service::DownloadService>,
}

impl ServiceLayer {
    pub fn new(repositories: Arc<repositories::RepositoryLayer>) -> Self {
        // Resource services
        let track_service = Arc::new(track_service::TrackService::new(
            repositories.track.clone(),
            repositories.album.clone(),
            repositories.artist.clone(),
        ));
        let album_service = Arc::new(album_service::AlbumService::new(
            repositories.album.clone(),
            repositories.artist.clone(),
        ));
        let artist_service = Arc::new(artist_service::ArtistService::new(
            repositories.artist.clone(),
        ));
        let playlist_service = Arc::new(playlist_service::PlaylistService::new(
            repositories.playlist.clone(),
        ));
        let sync_schedule_service = Arc::new(sync_schedule_service::SyncScheduleService::new(
            repositories.sync_schedule.clone(),
        ));
        let task_service = Arc::new(task_service::TaskService::new(repositories.task.clone()));

        // Services
        let download_service = Arc::new(download_service::DownloadService::new(
            track_service.clone(),
            album_service.clone(),
            artist_service.clone(),
            playlist_service.clone(),
            task_service.clone(),
        ));

        Self {
            track_service,
            album_service,
            artist_service,
            playlist_service,
            sync_schedule_service,
            task_service,

            download_service,
        }
    }
}
