use std::sync::Arc;

use diesel::SqliteConnection;

use crate::ports::repositories::PlaylistRepository;

pub struct PlaylistService {
    playlist_repo: Arc<dyn PlaylistRepository + Send + Sync>,
}

impl PlaylistService {
    pub fn new(playlist_repo: Arc<dyn PlaylistRepository + Send + Sync>) -> Self {
        Self { playlist_repo }
    }

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> shared::types::SoundomeResult<shared::models::Playlist> {
        self.playlist_repo.get_by_id(conn, id)
    }

    pub fn get_by_source_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<shared::models::Playlist> {
        self.playlist_repo.get_by_source_url(conn, url).ok().flatten()
    }

    /// Create a new playlist or, if one already exists for this `source_url`, update its sync timestamp.
    pub fn upsert(&self, conn: &mut SqliteConnection, playlist: &shared::models::Playlist) -> shared::types::SoundomeResult<shared::models::Playlist> {
        if let Some(url) = &playlist.source_url {
            if let Some(existing) = self.get_by_source_url(conn, url) {
                let id = existing.id.expect("persisted playlist must have an id");
                self.playlist_repo.update_last_sync(conn, id)?;
                return Ok(existing);
            }
        }
        self.playlist_repo.create(conn, playlist)
    }

    /// Link a track to a playlist. Silently ignores duplicates.
    pub fn add_track(&self, conn: &mut SqliteConnection, playlist_id: i32, track_id: i32, position: Option<i32>) -> shared::types::SoundomeResult<()> {
        self.playlist_repo.add_track(conn, playlist_id, track_id, position)
    }
}
