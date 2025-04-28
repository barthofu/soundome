use diesel::SqliteConnection;

use crate::ports::repositories::ArtistRepository;

pub struct ArtistService {
    artist_repo: Box<dyn ArtistRepository + Send + Sync>,
}

impl ArtistService {
    pub fn new(
        artist_repo: Box<dyn ArtistRepository + Send + Sync>,
    ) -> Self {
        Self { 
            artist_repo,
        }
    }

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> shared::types::SoundomeResult<shared::models::Artist> {
        self.artist_repo.get_by_id(conn, id)
    }

    pub fn create(&self, conn: &mut SqliteConnection, new_artist: &shared::models::Artist) -> shared::types::SoundomeResult<shared::models::Artist> {
        self.artist_repo.create(conn, new_artist)
    }

    pub fn update(&self, conn: &mut SqliteConnection, id: i32, updated_artist: &shared::models::Artist) -> shared::types::SoundomeResult<shared::models::Artist> {
        self.artist_repo.update(conn, id, updated_artist)
    }
}
