use std::{path::PathBuf, sync::Arc};

use config::Config;
use diesel::SqliteConnection;

use crate::ports::repositories::{AlbumRepository, ArtistRepository, PlaylistRepository, TrackRepository};
use crate::services::resources::track_ops::delete_track_with_cascade;

pub struct PlaylistService {
    playlist_repo: Arc<dyn PlaylistRepository + Send + Sync>,
    track_repo: Arc<dyn TrackRepository + Send + Sync>,
    album_repo: Arc<dyn AlbumRepository + Send + Sync>,
    artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
}

impl PlaylistService {
    pub fn new(
        playlist_repo: Arc<dyn PlaylistRepository + Send + Sync>,
        track_repo: Arc<dyn TrackRepository + Send + Sync>,
        album_repo: Arc<dyn AlbumRepository + Send + Sync>,
        artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
    ) -> Self {
        Self {
            playlist_repo,
            track_repo,
            album_repo,
            artist_repo,
        }
    }

    pub fn get_all(
        &self,
        conn: &mut SqliteConnection,
    ) -> shared::types::SoundomeResult<Vec<shared::models::Playlist>> {
        self.playlist_repo.get_all(conn)
    }

    pub fn get_by_id(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> shared::types::SoundomeResult<shared::models::Playlist> {
        self.playlist_repo.get_by_id(conn, id)
    }

    pub fn get_by_source_url(
        &self,
        conn: &mut SqliteConnection,
        url: &str,
    ) -> Option<shared::models::Playlist> {
        self.playlist_repo
            .get_by_source_url(conn, url)
            .ok()
            .flatten()
    }

    /// Create a new playlist or, if one already exists for this `source_url`, update its sync timestamp.
    pub fn upsert(
        &self,
        conn: &mut SqliteConnection,
        playlist: &shared::models::Playlist,
    ) -> shared::types::SoundomeResult<shared::models::Playlist> {
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
    pub fn add_track(
        &self,
        conn: &mut SqliteConnection,
        playlist_id: i32,
        track_id: i32,
        position: Option<i32>,
    ) -> shared::types::SoundomeResult<()> {
        self.playlist_repo
            .add_track(conn, playlist_id, track_id, position)
    }

    /// Return all finalized tracks in the playlist, ordered by position.
    pub fn get_tracks(
        &self,
        conn: &mut SqliteConnection,
        playlist_id: i32,
    ) -> shared::types::SoundomeResult<Vec<shared::models::Track>> {
        self.playlist_repo.get_tracks(conn, playlist_id)
    }

    /// Regenerate the M3U8 file for a playlist from the current DB state.
    ///
    /// The output directory is taken from `Config::get().playlists.m3u8_dir`; when
    /// absent it defaults to `{base_library_dir}/.playlists/`.
    pub fn export_m3u8(
        &self,
        conn: &mut SqliteConnection,
        playlist_id: i32,
    ) -> shared::types::SoundomeResult<PathBuf> {
        let playlist = self.playlist_repo.get_by_id(conn, playlist_id)?;
        let tracks = self.playlist_repo.get_tracks(conn, playlist_id)?;

        let cfg = Config::get();
        let output_dir = Self::resolve_m3u8_dir(cfg);

        organizer::playlist_writer::write_m3u8(&playlist, &tracks, &output_dir)
    }

    /// Resolve the M3U8 output directory from config, falling back to
    /// `{base_library_dir}/.playlists/` when not explicitly configured.
    pub fn resolve_m3u8_dir(cfg: &Config) -> PathBuf {
        match &cfg.playlists.m3u8_dir {
            Some(dir) => PathBuf::from(dir),
            None => PathBuf::from(&cfg.general.base_library_dir).join(".playlists"),
        }
    }

    pub fn count(&self, conn: &mut SqliteConnection) -> shared::types::SoundomeResult<i64> {
        self.playlist_repo.count(conn)
    }

    /// Delete a playlist and its track associations.
    ///
    /// When `delete_tracks` is `true`, every track that belongs to this playlist
    /// is also deleted from the library via `delete_track_with_cascade`, which
    /// automatically removes orphaned albums and artists.
    pub fn delete_by_id(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        delete_tracks: bool,
    ) -> shared::types::SoundomeResult<()> {
        if delete_tracks {
            let tracks = self.playlist_repo.get_tracks(conn, id)?;
            // Remove playlist + junction rows first so they don't block track deletion.
            self.playlist_repo.delete(conn, id)?;
            for track in tracks {
                if let Some(track_id) = track.id {
                    delete_track_with_cascade(
                        conn,
                        track_id,
                        &self.track_repo,
                        &self.album_repo,
                        &self.artist_repo,
                    )?;
                }
            }
        } else {
            self.playlist_repo.delete(conn, id)?;
        }
        Ok(())
    }
}
