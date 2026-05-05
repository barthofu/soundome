use std::sync::Arc;

use diesel::SqliteConnection;
use shared::{models::{Album, Artist, Playlist, SyncSchedule, Task, Track}, types::SoundomeResult};

pub struct RepositoryLayer {
    pub track: Arc<dyn TrackRepository>,
    pub album: Arc<dyn AlbumRepository>,
    pub artist: Arc<dyn ArtistRepository>,
    pub playlist: Arc<dyn PlaylistRepository>,
    pub task: Arc<dyn TaskRepository>,
    pub sync_schedule: Arc<dyn SyncScheduleRepository>,
}

// ================================================================================================

pub trait TrackRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Track>;
    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>>;
    fn create(&self, conn: &mut SqliteConnection, new_track: &Track) -> SoundomeResult<Track>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &Track) -> SoundomeResult<Track>;
    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;

    fn get_recent(&self, conn: &mut SqliteConnection, limit: i64) -> SoundomeResult<Vec<Track>>;
    fn get_pending_validations(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>>;
    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Track>;
    fn create_references(&self, conn: &mut SqliteConnection, track_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    /// Replace all references for a track (delete existing, then insert provided ones)
    fn set_references(&self, conn: &mut SqliteConnection, track_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()>;
    // /// Find a track by unique fields (e.g. title + artists + album)
    // fn find_by_unique_fields(&self, conn: &mut SqliteConnection, track: &Track) -> SoundomeResult<Option<Track>>;
}

pub trait AlbumRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Album>;
    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Album>>;
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
    /// Merge all source artists into `target_id`: re-point tracks, albums, and references, then delete sources.
    fn merge_into(&self, conn: &mut SqliteConnection, source_ids: &[i32], target_id: i32) -> SoundomeResult<()>;
    // /// Find an artist by unique fields (e.g. name)
    // fn find_by_unique_fields(&self, conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Option<Artist>>;
}

// ================================================================================================

pub trait PlaylistRepository: Send + Sync {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Playlist>;
    /// Returns `None` if no playlist with this source URL exists yet.
    fn get_by_source_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Option<Playlist>>;
    fn create(&self, conn: &mut SqliteConnection, playlist: &Playlist) -> SoundomeResult<Playlist>;
    fn update_last_sync(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;
    /// Link a track to a playlist. Silently ignores duplicate entries.
    fn add_track(&self, conn: &mut SqliteConnection, playlist_id: i32, track_id: i32, position: Option<i32>) -> SoundomeResult<()>;
}

// ================================================================================================

pub trait TaskRepository: Send + Sync {
    fn create(&self, conn: &mut SqliteConnection, task: &Task) -> SoundomeResult<Task>;
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Task>;
    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Task>>;
    fn set_running(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;
    fn update_progress(&self, conn: &mut SqliteConnection, id: i32, progress: i32, total: i32) -> SoundomeResult<()>;
    fn set_completed(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;
    fn set_failed(&self, conn: &mut SqliteConnection, id: i32, error: &str) -> SoundomeResult<()>;
    fn set_cancelled(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;
    fn get_by_status(&self, conn: &mut SqliteConnection, status: &str) -> SoundomeResult<Vec<Task>>;
    fn reset_for_retry(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;
}

// ================================================================================================

pub trait SyncScheduleRepository: Send + Sync {
    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<SyncSchedule>>;
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<SyncSchedule>;
    fn create(&self, conn: &mut SqliteConnection, schedule: &SyncSchedule) -> SoundomeResult<SyncSchedule>;
    fn update(&self, conn: &mut SqliteConnection, id: i32, schedule: &SyncSchedule) -> SoundomeResult<SyncSchedule>;
    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;
    /// Returns all schedules that are enabled and whose next_run is in the past (or NULL).
    fn get_due(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<SyncSchedule>>;
    /// Record that a schedule ran now and compute the next_run time.
    fn mark_ran(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()>;
}
