use std::sync::Arc;

use diesel::SqliteConnection;
use shared::{models::{Album, Artist, Track}, types::SoundomeResult};

pub struct RepositoryLayer {
    pub track: Arc<dyn TrackRepository>,
    pub album: Arc<dyn AlbumRepository>,
    pub artist: Arc<dyn ArtistRepository>,
}

impl RepositoryLayer {
    pub fn new(
        track: Arc<dyn TrackRepository>,
        album: Arc<dyn AlbumRepository>,
        artist: Arc<dyn ArtistRepository>,
    ) -> Self {
        Self { track, album, artist }
    }
}

// ================================================================================================

pub trait TrackRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Track>;
    fn create(&self, conn: &mut SqliteConnection, new_track: &Track) -> SoundomeResult<Track>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &Track) -> SoundomeResult<Track>;
}

pub trait AlbumRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Album>;
    fn create(&self, conn: &mut SqliteConnection, new_album: &Album) -> SoundomeResult<Album>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_album: &Album) -> SoundomeResult<Album>;
}

pub trait ArtistRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Artist>;
    fn create(&self, conn: &mut SqliteConnection, new_artist: &Artist) -> SoundomeResult<Artist>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_artist: &Artist) -> SoundomeResult<Artist>;
}