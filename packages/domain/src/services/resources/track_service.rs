use std::sync::Arc;

use diesel::SqliteConnection;

use crate::ports::repositories::{AlbumRepository, ArtistRepository, TrackRepository};

pub struct TrackService {
    track_repo: Arc<dyn TrackRepository + Send + Sync>,
    album_repo: Arc<dyn AlbumRepository + Send + Sync>,
    artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
}

impl TrackService {
    pub fn new(
        track_repo: Arc<dyn TrackRepository + Send + Sync>,
        album_repo: Arc<dyn AlbumRepository + Send + Sync>,
        artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
    ) -> Self {
        Self {
            track_repo,
            album_repo,
            artist_repo,
        }
    }

    // CRUD

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> shared::types::SoundomeResult<shared::models::Track> {
        self.track_repo.get_by_id(conn, id)
    }

    pub fn create(&self, conn: &mut SqliteConnection, new_track: &shared::models::Track) -> shared::types::SoundomeResult<shared::models::Track> {
        self.track_repo.create(conn, new_track)
    }

    pub fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &shared::models::Track) -> shared::types::SoundomeResult<shared::models::Track> {
        self.track_repo.update(conn, id, updated_track)
    }

    // Getters

    pub fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<shared::models::Track> {
        self.track_repo.get_by_url(conn, url).ok()
    }
}
