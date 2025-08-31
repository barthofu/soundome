use std::sync::Arc;

use diesel::SqliteConnection;

use crate::ports::repositories::AlbumRepository;

pub struct AlbumService {
    album_repo: Arc<dyn AlbumRepository + Send + Sync>,
}

impl AlbumService {
    pub fn new(
        album_repo: Arc<dyn AlbumRepository + Send + Sync>,
    ) -> Self {
        Self {
            album_repo,
        }
    }

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> shared::types::SoundomeResult<shared::models::Album> {
        self.album_repo.get_by_id(conn, id)
    }

    pub fn create(&self, conn: &mut SqliteConnection, new_album: &shared::models::Album) -> shared::types::SoundomeResult<shared::models::Album> {
        self.album_repo.create(conn, new_album)
    }

    pub fn update(&self, conn: &mut SqliteConnection, id: i32, updated_album: &shared::models::Album) -> shared::types::SoundomeResult<shared::models::Album> {
        self.album_repo.update(conn, id, updated_album)
    }

    pub fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<shared::models::Album> {
        self.album_repo.get_by_url(conn, url).ok()
    }
}
