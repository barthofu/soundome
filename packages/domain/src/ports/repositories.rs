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
    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Track>;
    fn create_references(&self, conn: &mut SqliteConnection, track_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    /// Replace all references for a track (delete existing, then insert provided ones)
    fn set_references(&self, conn: &mut SqliteConnection, track_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    // /// Find a track by unique fields (e.g. title + artists + album)
    // fn find_by_unique_fields(&self, conn: &mut SqliteConnection, track: &Track) -> SoundomeResult<Option<Track>>;
}

pub trait AlbumRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Album>;
    fn create(&self, conn: &mut SqliteConnection, new_album: &Album) -> SoundomeResult<Album>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_album: &Album) -> SoundomeResult<Album>;
    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Album>;
    fn create_references(&self, conn: &mut SqliteConnection, album_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    fn create_or_ignore(&self, conn: &mut SqliteConnection, album: &Album) -> SoundomeResult<Album>;
    /// Replace all references for an album (delete existing, then insert provided ones)
    fn set_references(&self, conn: &mut SqliteConnection, album_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    // /// Find an album by unique fields (e.g. title + artists + date)
    // fn find_by_unique_fields(&self, conn: &mut SqliteConnection, album: &Album) -> SoundomeResult<Option<Album>>;
}

pub trait ArtistRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Artist>;
    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Artist>>;
    fn create(&self, conn: &mut SqliteConnection, new_artist: &Artist) -> SoundomeResult<Artist>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_artist: &Artist) -> SoundomeResult<Artist>;
    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Artist>;
    fn create_references(&self, conn: &mut SqliteConnection, artist_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    fn create_track_relationship(&self, conn: &mut SqliteConnection, artist_id: i32, track_id: i32) -> SoundomeResult<()>;
    fn create_album_relationship(&self, conn: &mut SqliteConnection, artist_id: i32, album_id: i32) -> SoundomeResult<()>;
    fn create_or_ignore(&self, conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Artist>;
    /// Replace all references for an artist (delete existing, then insert provided ones)
    fn set_references(&self, conn: &mut SqliteConnection, artist_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    /// Replace all artists attached to a given track
    fn set_track_artists(&self, conn: &mut SqliteConnection, track_id: i32, artist_ids: &[i32]) -> SoundomeResult<()>;
    /// Replace all artists attached to a given album
    fn set_album_artists(&self, conn: &mut SqliteConnection, album_id: i32, artist_ids: &[i32]) -> SoundomeResult<()>;
    // /// Find an artist by unique fields (e.g. name)
    // fn find_by_unique_fields(&self, conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Option<Artist>>;
}