use std::sync::Arc;

use diesel::SqliteConnection;
use shared::{models::{Album, Artist, Track}, types::SoundomeResult};

pub struct RepositoryLayer {
    pub track: Arc<dyn TrackRepository>,
    pub album: Arc<dyn AlbumRepository>,
    pub artist: Arc<dyn ArtistRepository>,
}

// ================================================================================================

pub trait TrackRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Track>;
    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>>;
    fn create(&self, conn: &mut SqliteConnection, new_track: &Track) -> SoundomeResult<Track>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &Track) -> SoundomeResult<Track>;

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Track>;
    fn create_references(&self, conn: &mut SqliteConnection, track_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
}

pub trait AlbumRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Album>;
    fn create(&self, conn: &mut SqliteConnection, new_album: &Album) -> SoundomeResult<Album>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_album: &Album) -> SoundomeResult<Album>;

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Album>;
    fn create_references(&self, conn: &mut SqliteConnection, album_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    fn create_or_ignore(&self, conn: &mut SqliteConnection, album: &Album) -> SoundomeResult<Album>;
}

pub trait ArtistRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Artist>;
    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Artist>>;
    fn create(&self, conn: &mut SqliteConnection, new_artist: &Artist) -> SoundomeResult<Artist>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_artist: &Artist) -> SoundomeResult<Artist>;

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Artist>;
    fn create_references(&self, conn: &mut SqliteConnection, artist_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    fn create_track_relationship(&self, conn: &mut SqliteConnection, artist_id: i32, track_id: i32) -> SoundomeResult<()>;
    fn create_album_relationship(&self, conn: &mut SqliteConnection, artist_id: i32, album_id: i32) -> SoundomeResult<()>;
    fn create_or_ignore(&self, conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Artist>;
}