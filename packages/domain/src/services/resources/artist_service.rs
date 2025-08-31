use std::sync::Arc;

use diesel::SqliteConnection;

use crate::ports::repositories::ArtistRepository;

pub struct ArtistService {
    artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
}

impl ArtistService {
    pub fn new(
        artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
    ) -> Self {
        Self { 
            artist_repo,
        }
    }

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> shared::types::SoundomeResult<shared::models::Artist> {
        self.artist_repo.get_by_id(conn, id)
    }

    pub fn get_all(&self, conn: &mut SqliteConnection) -> shared::types::SoundomeResult<Vec<shared::models::Artist>> {
        self.artist_repo.get_all(conn)
    }

    pub fn create(&self, conn: &mut SqliteConnection, new_artist: &shared::models::Artist) -> shared::types::SoundomeResult<shared::models::Artist> {
        self.artist_repo.create(conn, new_artist)
    }

    pub fn update(&self, conn: &mut SqliteConnection, id: i32, updated_artist: &shared::models::Artist) -> shared::types::SoundomeResult<shared::models::Artist> {
        self.artist_repo.update(conn, id, updated_artist)
    }

    pub fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<shared::models::Artist> {
        self.artist_repo.get_by_url(conn, url).ok()
    }
}
