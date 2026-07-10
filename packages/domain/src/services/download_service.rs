use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

use config::Config;
use diesel::SqliteConnection;
use fetcher::{curate_source_url, Fetcher, Source};
use shared::models::ReferenceType;
use shared::{
    errors::Error,
    models::{Album, AlbumType, Artist, Platform, Playlist, Reference, TaskTrackValidation, Track},
    types::SoundomeResult,
    utils::enums::Match,
};
use uuid::Uuid;

use super::{
    album_service::AlbumService,
    artist_service::ArtistService,
    playlist_service::PlaylistService,
    task_service::TaskService,
    track_service::{TrackService, ValidationPatch},
};
pub use tagger::enricher::MatchCandidate;

pub struct DownloadService {
    track_service: Arc<TrackService>,
    album_service: Arc<AlbumService>,
    artist_service: Arc<ArtistService>,
    playlist_service: Arc<PlaylistService>,
    task_service: Arc<TaskService>,
}

// TODO: manage "to validate" tracks
impl DownloadService {
    pub fn new(
        track_service: Arc<TrackService>,
        album_service: Arc<AlbumService>,
        artist_service: Arc<ArtistService>,
        playlist_service: Arc<PlaylistService>,
        task_service: Arc<TaskService>,
    ) -> Self {
        Self {
            track_service,
            album_service,
            artist_service,
            playlist_service,
            task_service,
        }
    }

    /// Main entry point for downloading a track from a given URL (from any supported platform)
    pub async fn download_track_from_url(
        &self,
        url: &str,
        conn: &mut SqliteConnection,
    ) -> SoundomeResult<Track> {
        // Strip tracking/share query params (e.g. `si`, `utm_*`) so two submissions
        // of the same link that only differ by tracking noise dedupe correctly
        // against the `external_url` check right below.
        let curated_url = curate_source_url(url);
        let url = curated_url.as_str();

        tracing::info!("===========\nDownloading track from {:?}\n------", url);

        // Check if track already exists in DB
        if let Some(t) = self.track_service.get_by_url(conn, url) {
            return Err(Error::TrackExists(t.display()));
        }

        let fetcher = Fetcher::new().await;

        // Fetch track info from URL
        let mut track = fetcher.get_track_from_url(url).await?;
        fetcher.clean_track_metadata(&mut track).await?;
        tracing::info!(
            "Fetched track info from {}: {}",
            track.get_source_platform().as_ref(),
            track.display()
        );

        // Orchestrator workflow
        let final_track = self.orchestrator_workflow(conn, track).await?;
        Ok(final_track)
    }

    /// Main entry point for downloading a playlist from a given URL (from any supported platform).
    /// `task_id` is optional; when provided, progress is persisted to the task table in real-time.
    pub async fn sync_playlist_from_url(
        &self,
        url: &str,
        conn: &mut SqliteConnection,
        task_id: Option<i32>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> SoundomeResult<Vec<Track>> {
        // Strip tracking/share query params so two syncs of "the same" playlist
        // link (e.g. with vs without `?si=...&utm_source=...`) curate to the same
        // `source_url` instead of `PlaylistService::upsert` creating a duplicate
        // playlist row.
        let curated_url = curate_source_url(url);
        let url = curated_url.as_str();

        tracing::info!(
            "====================\nDownloading playlist from {:?}\n---------",
            url
        );

        let fetcher = Fetcher::new().await;

        // Fetch playlist metadata and upsert in DB
        let playlist_meta = fetcher
            .get_playlist_from_url(url)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Could not fetch playlist metadata ({}), using URL as name",
                    e
                );
                // Extract platform from URL for better fallback naming. "music.youtube.com"
                // must be checked before the plain "youtube.com" branch, since it also
                // contains "youtube.com" as a substring.
                let (platform, name) = if url.contains("spotify.com") {
                    (shared::models::Platform::Spotify, url.to_string())
                } else if url.contains("soundcloud.com") {
                    (shared::models::Platform::SoundCloud, url.to_string())
                } else if url.contains("music.youtube.com") {
                    (shared::models::Platform::YoutubeMusic, url.to_string())
                } else if url.contains("youtube.com") || url.contains("youtu.be") {
                    (shared::models::Platform::Youtube, url.to_string())
                } else {
                    (shared::models::Platform::Unknown, url.to_string())
                };
                shared::models::Playlist {
                    id: None,
                    name,
                    source: platform,
                    source_url: Some(url.to_string()),
                    cover: None,
                }
            });
        let playlist = self.playlist_service.upsert(conn, &playlist_meta)?;
        let playlist_id = playlist.id.expect("persisted playlist must have an id");
        tracing::info!(
            "Playlist upserted in DB: \"{}\" (id={})",
            playlist.name,
            playlist_id
        );

        // Update task label to the actual playlist name with platform indicator.
        if let Some(tid) = task_id {
            let label = format!("[{}] {}", playlist.source, playlist.name);
            if let Err(e) = self.task_service.update_label(conn, tid, &label) {
                tracing::warn!("Failed to update task label to playlist name: {}", e);
            }
        }

        let playlist_tracks = fetcher.get_playlist_tracks_from_url(url).await?;
        let total_tracks = playlist_tracks.len();
        tracing::info!("Found {} tracks in playlist", total_tracks);

        // Filter out existing tracks (link them to the playlist anyway) and collect new ones
        let mut new_tracks: Vec<(Option<i32>, Track)> = Vec::new();
        let mut stats = shared::models::TaskStats::default();
        for pt in &playlist_tracks {
            let track = &pt.track;
            let track_url = track
                .get_source()
                .and_then(|s| s.external_url.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let position = pt.position.map(|p| p as i32);
            if let Some(existing) = self.track_service.get_by_url(conn, &track_url) {
                tracing::warn!(
                    "   -> Track already exists in DB, linking to playlist: {}",
                    track.display()
                );
                let track_id = existing.id.expect("persisted track must have an id");
                if let Err(e) =
                    self.playlist_service
                        .add_track(conn, playlist_id, track_id, position)
                {
                    tracing::error!(
                        "Failed to link existing track {} to playlist: {}",
                        track_id,
                        e
                    );
                }
                stats.skipped += 1;
                if let Some(tid) = task_id {
                    let current = stats.skipped;
                    if let Err(e) =
                        self.task_service
                            .update_progress(conn, tid, current, total_tracks as i32)
                    {
                        tracing::warn!("Failed to update task progress: {}", e);
                    }
                    if let Err(e) = self.task_service.update_stats(conn, tid, &stats) {
                        tracing::warn!("Failed to update task stats: {}", e);
                    }
                }
            } else {
                new_tracks.push((position, track.clone()));
            }
        }

        tracing::info!(
            "{} new tracks to download after filtering existing ones",
            new_tracks.len()
        );

        // Clean metadata for all new tracks
        let mut new_track_values: Vec<Track> = new_tracks.iter().map(|(_, t)| t.clone()).collect();
        self.clean_tracks_metadata_with_progress(
            &fetcher,
            conn,
            &mut new_track_values,
            task_id,
            &mut stats,
        )
        .await;

        // Process each new track and link it to the playlist
        let mut new_processed_tracks = Vec::new();
        for (i, (position, _)) in new_tracks.iter().enumerate() {
            // Check for cancellation before processing next track
            if cancel_flag
                .as_ref()
                .is_some_and(|f| f.load(Ordering::Relaxed))
            {
                tracing::info!(
                    "Playlist sync cancelled after processing {}/{} new tracks",
                    i,
                    new_tracks.len()
                );
                return Err(Error::Cancelled);
            }

            let track = &new_track_values[i];
            tracing::info!("Processing track: {}", track.display());
            match self.orchestrator_workflow(conn, track.clone()).await {
                Ok(t) => {
                    tracing::info!("Successfully processed track: {}", t.display());
                    if t.needs_validation {
                        stats.to_validate += 1;
                        stats.to_validate_tracks.push(TaskTrackValidation {
                            track: t.display(),
                            track_id: t.id,
                            reason: t.validation_reason.clone(),
                        });
                    } else {
                        stats.downloaded += 1;
                    }
                    if let Some(track_id) = t.id {
                        if let Err(e) =
                            self.playlist_service
                                .add_track(conn, playlist_id, track_id, *position)
                        {
                            tracing::error!(
                                "Failed to link new track {} to playlist: {}",
                                track_id,
                                e
                            );
                        }
                    }
                    new_processed_tracks.push(t);
                }
                Err(e) => {
                    stats.errors.push(shared::models::TaskTrackError {
                        track: track.display(),
                        reason: e.to_string(),
                        track_id: None,
                        provider_url: track.get_provider().and_then(|p| p.external_url.clone()),
                    });
                    tracing::error!("Error downloading track {}: {:?}", track.display(), e);
                }
            }
            if let Some(tid) = task_id {
                let current = stats.skipped + (i as i32) + 1;
                if let Err(e) =
                    self.task_service
                        .update_progress(conn, tid, current, total_tracks as i32)
                {
                    tracing::warn!("Failed to update task progress: {}", e);
                }
                if let Err(e) = self.task_service.update_stats(conn, tid, &stats) {
                    tracing::warn!("Failed to update task stats: {}", e);
                }
            }
        }

        tracing::info!(
            "Playlist \"{}\": {} downloaded, {} to validate, {} skipped, {} errors (total {})",
            playlist.name,
            stats.downloaded,
            stats.to_validate,
            stats.skipped,
            stats.errors.len(),
            total_tracks,
        );

        // Best-effort: export updated playlist as an M3U8 file.
        self.export_playlist_m3u8(conn, &playlist, playlist_id);

        Ok(new_processed_tracks)
    }

    /// Main entry point for downloading/syncing all tracks from an artist URL.
    /// `task_id` is optional; when provided, progress is persisted to the task table in real-time.
    pub async fn sync_artist_from_url(
        &self,
        url: &str,
        conn: &mut SqliteConnection,
        task_id: Option<i32>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> SoundomeResult<Vec<Track>> {
        // Strip tracking/share query params for consistency with the other
        // `*_from_url` entry points (see `sync_playlist_from_url`).
        let curated_url = curate_source_url(url);
        let url = curated_url.as_str();

        tracing::info!(
            "====================\nSyncing artist from {:?}\n---------",
            url
        );

        let fetcher = Fetcher::new().await;

        // Fetch artist metadata and upsert in DB
        let artist_meta = fetcher.get_artist_from_url(url).await?;
        let artist = self.artist_service.create_or_ignore(conn, &artist_meta)?;
        let artist_id = artist.id.expect("persisted artist must have an id");
        tracing::info!(
            "Artist upserted in DB: \"{}\" (id={})",
            artist.name,
            artist_id
        );

        // Update task label to the artist name with platform indicator.
        if let Some(tid) = task_id {
            let platform = artist
                .get_source()
                .map(|r| r.platform.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            let label = format!("[{}] {}", platform, artist.name);
            if let Err(e) = self.task_service.update_label(conn, tid, &label) {
                tracing::warn!("Failed to update task label to artist name: {}", e);
            }
        }

        // Fetch all tracks from this artist
        let artist_tracks = fetcher.get_artist_tracks_from_url(url).await?;
        let total_tracks = artist_tracks.len();
        tracing::info!("Found {} tracks for artist", total_tracks);

        // Filter out existing tracks and collect new ones
        let mut new_tracks: Vec<Track> = Vec::new();
        let mut stats = shared::models::TaskStats::default();
        for track in &artist_tracks {
            let track_url = track
                .get_source()
                .and_then(|s| s.external_url.clone())
                .unwrap_or_else(|| "unknown".to_string());
            if self.track_service.get_by_url(conn, &track_url).is_some() {
                tracing::warn!("   -> Track already exists in DB: {}", track.display());
                stats.skipped += 1;
                if let Some(tid) = task_id {
                    let current = stats.skipped;
                    if let Err(e) =
                        self.task_service
                            .update_progress(conn, tid, current, total_tracks as i32)
                    {
                        tracing::warn!("Failed to update task progress: {}", e);
                    }
                    if let Err(e) = self.task_service.update_stats(conn, tid, &stats) {
                        tracing::warn!("Failed to update task stats: {}", e);
                    }
                }
            } else {
                new_tracks.push(track.clone());
            }
        }

        tracing::info!(
            "{} new tracks to download after filtering existing ones",
            new_tracks.len()
        );

        // Clean metadata for all new tracks
        self.clean_tracks_metadata_with_progress(
            &fetcher,
            conn,
            &mut new_tracks,
            task_id,
            &mut stats,
        )
        .await;

        // Process each new track
        let mut new_processed_tracks = Vec::new();
        for (i, track) in new_tracks.iter().enumerate() {
            // Check for cancellation before processing next track
            if cancel_flag
                .as_ref()
                .is_some_and(|f| f.load(Ordering::Relaxed))
            {
                tracing::info!(
                    "Artist sync cancelled after processing {}/{} new tracks",
                    i,
                    new_tracks.len()
                );
                return Err(Error::Cancelled);
            }

            tracing::info!("Processing track: {}", track.display());
            match self.orchestrator_workflow(conn, track.clone()).await {
                Ok(t) => {
                    tracing::info!("Successfully processed track: {}", t.display());
                    if t.needs_validation {
                        stats.to_validate += 1;
                        stats.to_validate_tracks.push(TaskTrackValidation {
                            track: t.display(),
                            track_id: t.id,
                            reason: t.validation_reason.clone(),
                        });
                    } else {
                        stats.downloaded += 1;
                    }
                    new_processed_tracks.push(t);
                }
                Err(e) => {
                    stats.errors.push(shared::models::TaskTrackError {
                        track: track.display(),
                        reason: e.to_string(),
                        track_id: None,
                        provider_url: track.get_provider().and_then(|p| p.external_url.clone()),
                    });
                    tracing::error!("Error downloading track {}: {:?}", track.display(), e);
                }
            }
            if let Some(tid) = task_id {
                let current = stats.skipped + (i as i32) + 1;
                if let Err(e) =
                    self.task_service
                        .update_progress(conn, tid, current, total_tracks as i32)
                {
                    tracing::warn!("Failed to update task progress: {}", e);
                }
                if let Err(e) = self.task_service.update_stats(conn, tid, &stats) {
                    tracing::warn!("Failed to update task stats: {}", e);
                }
            }
        }

        tracing::info!(
            "Artist \"{}\": {} downloaded, {} to validate, {} skipped, {} errors (total {})",
            artist.name,
            stats.downloaded,
            stats.to_validate,
            stats.skipped,
            stats.errors.len(),
            total_tracks,
        );

        Ok(new_processed_tracks)
    }

    /// Main entry point for downloading/syncing all tracks from an album URL.
    /// `task_id` is optional; when provided, progress is persisted to the task table in real-time.
    pub async fn sync_album_from_url(
        &self,
        url: &str,
        conn: &mut SqliteConnection,
        task_id: Option<i32>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> SoundomeResult<Vec<Track>> {
        // Strip tracking/share query params for consistency with the other
        // `*_from_url` entry points (see `sync_playlist_from_url`).
        let curated_url = curate_source_url(url);
        let url = curated_url.as_str();

        tracing::info!(
            "====================\nSyncing album from {:?}\n---------",
            url
        );

        let fetcher = Fetcher::new().await;

        // Fetch album metadata
        let album_meta = fetcher.get_album_from_url(url).await?;
        tracing::info!(
            "Album: \"{}\" by {}",
            album_meta.title,
            album_meta
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Update task label to the album title with platform indicator.
        if let Some(tid) = task_id {
            let platform = album_meta
                .get_source()
                .map(|r| r.platform.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            let label = format!("[{}] {}", platform, album_meta.title);
            if let Err(e) = self.task_service.update_label(conn, tid, &label) {
                tracing::warn!("Failed to update task label to album title: {}", e);
            }
        }

        // Fetch all tracks from this album
        let album_tracks = fetcher.get_album_tracks_from_url(url).await?;
        let total_tracks = album_tracks.len();
        tracing::info!("Found {} tracks in album", total_tracks);

        // Filter out existing tracks and collect new ones
        let mut new_tracks: Vec<Track> = Vec::new();
        let mut stats = shared::models::TaskStats::default();
        for track in &album_tracks {
            let track_url = track
                .get_source()
                .and_then(|s| s.external_url.clone())
                .unwrap_or_else(|| "unknown".to_string());
            if self.track_service.get_by_url(conn, &track_url).is_some() {
                tracing::warn!("   -> Track already exists in DB: {}", track.display());
                stats.skipped += 1;
                if let Some(tid) = task_id {
                    let current = stats.skipped;
                    if let Err(e) =
                        self.task_service
                            .update_progress(conn, tid, current, total_tracks as i32)
                    {
                        tracing::warn!("Failed to update task progress: {}", e);
                    }
                    if let Err(e) = self.task_service.update_stats(conn, tid, &stats) {
                        tracing::warn!("Failed to update task stats: {}", e);
                    }
                }
            } else {
                new_tracks.push(track.clone());
            }
        }

        tracing::info!(
            "{} new tracks to download after filtering existing ones",
            new_tracks.len()
        );

        // Clean metadata for all new tracks
        self.clean_tracks_metadata_with_progress(
            &fetcher,
            conn,
            &mut new_tracks,
            task_id,
            &mut stats,
        )
        .await;

        // Process each new track
        let mut new_processed_tracks = Vec::new();
        for (i, track) in new_tracks.iter().enumerate() {
            // Check for cancellation before processing next track
            if cancel_flag
                .as_ref()
                .is_some_and(|f| f.load(Ordering::Relaxed))
            {
                tracing::info!(
                    "Album sync cancelled after processing {}/{} new tracks",
                    i,
                    new_tracks.len()
                );
                return Err(Error::Cancelled);
            }

            tracing::info!("Processing track: {}", track.display());
            match self.orchestrator_workflow(conn, track.clone()).await {
                Ok(t) => {
                    tracing::info!("Successfully processed track: {}", t.display());
                    if t.needs_validation {
                        stats.to_validate += 1;
                        stats.to_validate_tracks.push(TaskTrackValidation {
                            track: t.display(),
                            track_id: t.id,
                            reason: t.validation_reason.clone(),
                        });
                    } else {
                        stats.downloaded += 1;
                    }
                    new_processed_tracks.push(t);
                }
                Err(e) => {
                    stats.errors.push(shared::models::TaskTrackError {
                        track: track.display(),
                        reason: e.to_string(),
                        track_id: None,
                        provider_url: track.get_provider().and_then(|p| p.external_url.clone()),
                    });
                    tracing::error!("Error downloading track {}: {:?}", track.display(), e);
                }
            }
            if let Some(tid) = task_id {
                let current = stats.skipped + (i as i32) + 1;
                if let Err(e) =
                    self.task_service
                        .update_progress(conn, tid, current, total_tracks as i32)
                {
                    tracing::warn!("Failed to update task progress: {}", e);
                }
                if let Err(e) = self.task_service.update_stats(conn, tid, &stats) {
                    tracing::warn!("Failed to update task stats: {}", e);
                }
            }
        }

        tracing::info!(
            "Album \"{}\": {} downloaded, {} to validate, {} skipped, {} errors (total {})",
            album_meta.title,
            stats.downloaded,
            stats.to_validate,
            stats.skipped,
            stats.errors.len(),
            total_tracks,
        );

        Ok(new_processed_tracks)
    }

    /// Ingest all audio files found in `ingest_dir`, one by one.
    ///
    /// Progress and per-track stats are persisted live to `task_id` so the UI
    /// can poll `GET /api/tasks/:id` for real-time feedback.
    pub async fn ingest_local_dir(
        &self,
        conn: &mut SqliteConnection,
        ingest_dir: &Path,
        task_id: i32,
    ) -> SoundomeResult<()> {
        let audio_extensions = ["mp3", "flac", "m4a", "mp4", "aac", "ogg", "opus", "wav"];

        // Collect all audio files first so we know the total upfront.
        let files: Vec<PathBuf> = walkdir::WalkDir::new(ingest_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| audio_extensions.contains(&x.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        let total = files.len() as i32;
        tracing::info!("Ingest dir {:?}: found {} audio files", ingest_dir, total);

        let mut stats = shared::models::TaskStats::default();

        for (i, file_path) in files.iter().enumerate() {
            tracing::info!("Ingesting [{}/{}]: {:?}", i + 1, total, file_path);

            match self.ingest_local_file(conn, file_path).await {
                Ok(t) => {
                    if t.needs_validation {
                        stats.to_validate += 1;
                        stats
                            .to_validate_tracks
                            .push(shared::models::TaskTrackValidation {
                                track: t.display(),
                                track_id: t.id,
                                reason: t.validation_reason.clone(),
                            });
                    } else {
                        stats.downloaded += 1;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to ingest {:?}: {}", file_path, e);
                    stats.errors.push(shared::models::TaskTrackError {
                        track: file_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| file_path.display().to_string()),
                        reason: e.to_string(),
                        track_id: None,
                        provider_url: None,
                    });
                }
            }

            if let Err(e) = self
                .task_service
                .update_progress(conn, task_id, (i + 1) as i32, total)
            {
                tracing::warn!("Failed to update ingest task progress: {}", e);
            }
            if let Err(e) = self.task_service.update_stats(conn, task_id, &stats) {
                tracing::warn!("Failed to update ingest task stats: {}", e);
            }
        }

        tracing::info!(
            "Ingest dir complete: {} ingested, {} to validate, {} errors",
            stats.downloaded,
            stats.to_validate,
            stats.errors.len()
        );

        Ok(())
    }

    // ============================================================================================
    // == Local file ingest
    // ============================================================================================

    /// Ingest a single local audio file into the library.
    ///
    /// Workflow (mirrors `docs/workflows/download.md` — "Import a local file"):
    /// 1. Read tags from the file.
    /// 2. Evaluate metadata quality; enrich via MusicBrainz when needed.
    /// 3. If enrichment is partial or absent, persist as `needs_validation = true`.
    /// 4. Deduplicate by title/artist against existing DB tracks.
    /// 5. Tag, organise, and persist the winner.
    pub async fn ingest_local_file(
        &self,
        conn: &mut SqliteConnection,
        file_path: &Path,
    ) -> SoundomeResult<Track> {
        tracing::info!("===========\nIngesting local file: {:?}\n------", file_path);

        // Step 1: Read tags from the file.
        let mut track = tagger::file::get_track_from_file(&file_path.to_path_buf())
            .map_err(|e| Error::Custom(format!("Failed to read audio tags: {e}")))?;

        // Step 1b: If track_number is missing from the tags, try to infer it from
        // the file name. Many DIY releases use patterns like "08 - Title.flac" or
        // "08_Title.flac". Having the track number improves match scoring significantly.
        if track.track_number.is_none() {
            track.track_number = infer_track_number_from_filename(file_path);
            if let Some(n) = track.track_number {
                tracing::debug!("Inferred track_number {} from filename", n);
            }
        }

        tracing::info!("Read tags from file: {}", track.display());

        // Step 2: Enrich metadata using the ingest-specific provider order (Spotify first).
        // `enrich_metada` may set `needs_validation` on the track.
        let (should_validate, existing_track_opt) =
            self.enrich_metada(conn, &mut track, true).await?;

        if should_validate {
            tracing::warn!(
                "Ingest: saving for manual validation — reason={:?}",
                track.validation_reason
            );
            // Copy the file to the staging dir so it is not moved from its original location yet.
            let staged_path = self.stage_local_file(file_path)?;
            track.file_path = Some(staged_path);
            let saved = self.save_track(conn, &track).await?;
            return Ok(saved);
        }

        // Step 3: Deduplication.
        let existing_track = if existing_track_opt.is_some() {
            existing_track_opt
        } else {
            self.dedupe_track(conn, &track).await
        };

        match existing_track {
            Some(mut existing_track) => {
                tracing::info!(
                    "Ingest: existing track found: {}, comparing quality",
                    existing_track.display()
                );

                let new_is_better = self
                    .track_service
                    .is_better_quality(&existing_track, &track);

                if new_is_better {
                    tracing::info!("Ingest: new file has better quality, replacing");

                    let mut track_for_merge = track.clone();
                    normalize_album_and_artist_refs_as_metadata(&mut track_for_merge);
                    existing_track.transpose_refs(&track_for_merge);
                    apply_source_provider_replacement(&mut existing_track, &track);

                    self.process_track_file(&mut existing_track, file_path)
                        .await?;
                    let updated = self.save_track(conn, &existing_track).await?;
                    Ok(updated)
                } else {
                    tracing::info!(
                        "Ingest: existing file is equal or better quality, skipping file move"
                    );

                    // Keep existing audio; merge useful metadata from the ingested file.
                    let mut track_for_merge = track.clone();
                    normalize_album_and_artist_refs_as_metadata(&mut track_for_merge);
                    demote_track_source_and_provider_to_metadata(&mut track_for_merge);
                    existing_track.transpose_refs(&track_for_merge);

                    let updated = self.save_track(conn, &existing_track).await?;
                    Ok(updated)
                }
            }
            None => {
                tracing::info!("Ingest: no existing track, finalising");
                self.process_track_file(&mut track, file_path).await?;
                let inserted = self.save_track(conn, &track).await?;
                Ok(inserted)
            }
        }
    }

    /// Copy a local file into the staging directory so it can be processed without
    /// modifying the original location. Returns the path of the staged copy.
    ///
    /// The staged filename is prefixed with a UUID to guarantee uniqueness even when
    /// multiple files share the same original name (e.g. two different `track.mp3`
    /// from different ingest sessions).
    fn stage_local_file(&self, source: &Path) -> SoundomeResult<PathBuf> {
        let staging_dir = PathBuf::from(&Config::get().general.temp_download_dir);
        std::fs::create_dir_all(&staging_dir)
            .map_err(|e| Error::Custom(format!("Could not create staging dir: {e}")))?;

        let file_name = source
            .file_name()
            .ok_or_else(|| Error::Custom("Source path has no file name".to_string()))?
            .to_string_lossy();

        // Prefix with a UUID so two files named identically never collide in staging.
        let unique_name = format!("{}-{}", Uuid::new_v4(), file_name);
        let dest = staging_dir.join(&unique_name);

        std::fs::copy(source, &dest)
            .map_err(|e| Error::Custom(format!("Failed to stage local file: {e}")))?;
        tracing::debug!("Staged local file {:?} → {:?}", source, dest);
        Ok(dest)
    }

    // ============================================================================================
    // == Sub private and utils methods
    // ============================================================================================

    /// Re-query metadata providers for a pending track and return scored candidates.
    /// Used by the validation UI to show potential matches.
    pub async fn get_match_candidates(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> SoundomeResult<Vec<tagger::enricher::MatchCandidate>> {
        let track = self.track_service.get_by_id(conn, id)?;
        let candidates = tagger::enricher::get_candidates_for_track(&track).await;
        Ok(candidates)
    }

    /// Called after a user approves a pending validation through the web UI.
    ///
    /// The track already has an audio file in the staging folder (downloaded at fetch time).
    /// This method applies the optional metadata `patch`, tags the staged file, moves it
    /// to the library, and clears the validation flag.
    pub async fn finalize_validated_track(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        patch: ValidationPatch,
    ) -> SoundomeResult<Track> {
        // 1. Load current track from DB
        let mut track = self.track_service.get_by_id(conn, id)?;

        // 2. Apply metadata patch
        if let Some(title) = patch.title {
            track.title = title;
        }
        if let Some(genre) = patch.genre {
            track.genre = Some(genre);
        }
        if let Some(date) = patch.date {
            track.date = Some(date);
        }
        if let Some(tn) = patch.track_number {
            track.track_number = Some(tn);
        }
        if let Some(dn) = patch.disc_number {
            track.disc_number = Some(dn);
        }
        if let Some(label) = patch.label {
            track.label = Some(label);
        }

        if let Some(names) = patch.artists {
            let mut artists: Vec<Artist> = Vec::with_capacity(names.len());
            for name in names {
                let artist = Artist {
                    id: None,
                    name,
                    icon: None,
                    references: vec![],
                };
                let saved = self.artist_service.create_or_ignore(conn, &artist)?;
                artists.push(saved);
            }
            track.artists = artists;
        }

        if let Some(album_title) = patch.album_title {
            match track.album.as_mut() {
                Some(album) => album.title = album_title,
                None => {
                    track.album = Some(Album {
                        id: None,
                        title: album_title,
                        artists: vec![],
                        album_type: AlbumType::Album,
                        cover: None,
                        date: None,
                        references: vec![],
                    });
                }
            }
        }

        // 3. Resolve the audio file path: use the staged file if present, otherwise
        //    download from the provider URL supplied by the user (DRM fallback).
        let file_path = if let Some(staged) = track.file_path.clone() {
            staged
        } else {
            let provider_url = patch.provider_url.as_ref().ok_or_else(|| {
                Error::Custom(format!(
                    "track {} has no staged file and no provider_url was provided",
                    id
                ))
            })?;

            tracing::info!(
                "No staged file for track {} — downloading from provider: {}",
                id,
                provider_url
            );

            let provider_platform = if provider_url.contains("music.youtube.com") {
                Platform::YoutubeMusic
            } else {
                Platform::Youtube
            };

            let provider_ref = Reference {
                id: None,
                ref_type: ReferenceType::Provider,
                platform: provider_platform,
                external_id: None,
                external_url: Some(provider_url.clone()),
            };
            track.references.push(provider_ref.clone());

            let source_ref = track
                .get_source()
                .ok_or_else(|| Error::Custom(format!("track {} has no source reference", id)))?;

            let staging_dir = PathBuf::from(&Config::get().general.temp_download_dir);
            downloader::download(&source_ref, &provider_ref, &track.title, staging_dir).await?
        };
        track.file_path = Some(file_path.clone());
        self.process_track_file(&mut track, &file_path).await?;

        // 4. Clear validation flag and persist
        track.needs_validation = false;
        track.validation_reason = None;

        self.save_track(conn, &track).await
    }

    // ============================================================================================
    // == Sub private and utils methods (internal)
    // ============================================================================================

    /// Search YouTube and YouTube Music for provider candidates matching a pending track.
    /// Returns all results unfiltered so the user can select the correct video manually.
    /// Intended for DRM-protected SoundCloud tracks that could not be auto-downloaded.
    pub async fn get_youtube_provider_candidates(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> SoundomeResult<Vec<tagger::enricher::MatchCandidate>> {
        let track = self.track_service.get_by_id(conn, id)?;
        let results = downloader::search_youtube_candidates(&track).await?;

        let candidates = results
            .into_iter()
            .map(|t| {
                let provider = t
                    .get_provider()
                    .and_then(|r| r.external_url.clone())
                    .map(|u| {
                        if u.contains("music.youtube.com") {
                            "youtube_music"
                        } else {
                            "youtube"
                        }
                    })
                    .unwrap_or("youtube")
                    .to_string();
                tagger::enricher::MatchCandidate {
                    track: t,
                    score: 1.0,
                    provider,
                }
            })
            .collect();

        Ok(candidates)
    }

    // ============================================================================================
    // == Thumbnail-from-references (manual edit UI)
    // ============================================================================================

    /// Best-effort: resolve an artist's photo by re-querying whichever of its existing
    /// references point to a provider that exposes artist images (Spotify, SoundCloud,
    /// YouTube Music), then persist the first image found as the artist's `icon`.
    ///
    /// Used by the manual edit UI's "Fetch from references" action. Returns `Ok(None)`
    /// (not an error) when no reference resolves to an image, so the caller can tell
    /// "nothing found" apart from a network or database failure.
    pub async fn fetch_artist_icon_from_references(
        &self,
        conn: &mut SqliteConnection,
        artist_id: i32,
    ) -> SoundomeResult<Option<Artist>> {
        let mut artist = self.artist_service.get_by_id(conn, artist_id)?;
        let fetcher = Fetcher::new().await;

        let mut found_icon = None;
        for reference in &artist.references {
            let Some(url) = reference.external_url.as_deref() else {
                continue;
            };
            if !Fetcher::is_valid_artist_url(url) {
                continue;
            }
            match fetcher.get_artist_from_url(url).await {
                Ok(fetched) if fetched.icon.is_some() => {
                    found_icon = fetched.icon;
                    break;
                }
                Ok(_) => {}
                Err(e) => tracing::debug!(
                    "fetch_artist_icon_from_references: reference {} did not resolve: {}",
                    url,
                    e
                ),
            }
        }

        let Some(icon) = found_icon else {
            return Ok(None);
        };

        artist.icon = Some(icon);
        let saved = self.artist_service.update(conn, artist_id, &artist)?;
        Ok(Some(saved))
    }

    /// Same idea as `fetch_artist_icon_from_references`, but resolves and persists an
    /// album's `cover` instead.
    pub async fn fetch_album_cover_from_references(
        &self,
        conn: &mut SqliteConnection,
        album_id: i32,
    ) -> SoundomeResult<Option<Album>> {
        let mut album = self.album_service.get_by_id(conn, album_id)?;
        let fetcher = Fetcher::new().await;

        let mut found_cover = None;
        for reference in &album.references {
            let Some(url) = reference.external_url.as_deref() else {
                continue;
            };
            if !Fetcher::is_valid_album_url(url) {
                continue;
            }
            match fetcher.get_album_from_url(url).await {
                Ok(fetched) if fetched.cover.is_some() => {
                    found_cover = fetched.cover;
                    break;
                }
                Ok(_) => {}
                Err(e) => tracing::debug!(
                    "fetch_album_cover_from_references: reference {} did not resolve: {}",
                    url,
                    e
                ),
            }
        }

        let Some(cover) = found_cover else {
            return Ok(None);
        };

        album.cover = Some(cover);
        let saved = self.album_service.update(conn, album_id, &album)?;
        Ok(Some(saved))
    }

    /// Clean metadata for `tracks` via the fetcher, reporting live per-batch progress
    /// into `stats.ai_curation` (persisted through `task_id`) so the frontend can show
    /// an "AI curation in progress" indicator while SoundCloud batches are processed.
    async fn clean_tracks_metadata_with_progress(
        &self,
        fetcher: &Fetcher,
        conn: &mut SqliteConnection,
        tracks: &mut [Track],
        task_id: Option<i32>,
        stats: &mut shared::models::TaskStats,
    ) {
        if let Some(tid) = task_id {
            stats.ai_curation = Some(shared::models::AiCurationProgress {
                processed: 0,
                total: tracks.len() as i32,
            });
            if let Err(e) = self.task_service.update_stats(conn, tid, stats) {
                tracing::warn!("Failed to update task stats: {}", e);
            }
        }

        let mut on_batch = |processed: usize, total: usize| {
            if let Some(tid) = task_id {
                stats.ai_curation = Some(shared::models::AiCurationProgress {
                    processed: processed as i32,
                    total: total as i32,
                });
                if let Err(e) = self.task_service.update_stats(conn, tid, stats) {
                    tracing::warn!("Failed to update AI curation progress: {}", e);
                }
            }
        };

        if let Err(e) = fetcher
            .clean_tracks_metadata(
                &mut tracks.iter_mut().collect::<Vec<_>>(),
                Some(&mut on_batch),
            )
            .await
        {
            tracing::warn!("Failed to clean tracks title and artist name: {}", e);
        }

        if task_id.is_some() {
            stats.ai_curation = None;
        }
    }

    async fn orchestrator_workflow(
        &self,
        conn: &mut SqliteConnection,
        track: Track,
    ) -> SoundomeResult<Track> {
        let mut track = track;

        // Step 1: Enrich metadata
        tracing::info!("Getting metadata via tagger providers");
        let (should_validate, mut existing_track) =
            self.enrich_metada(conn, &mut track, false).await?;

        // Step 2: Try to download to staging.
        // SoundCloud DRM-protected tracks will return SoundCloudDrmProtected instead of a hard error.
        tracing::info!("Searching and downloading track from provider (staging)");
        let file_path_opt = match self.download_track(&mut track).await {
            Ok(path) => Some(path),
            Err(Error::SoundCloudDrmProtected(_)) => {
                tracing::warn!("SoundCloud track is DRM protected");

                // Before falling back to manual validation, check whether a Spotify metadata
                // reference is already attached (e.g. from enrichment) and, if so, retry
                // automatically via the existing Spotify → YouTube/YouTube Music matching
                // flow instead of immediately requiring manual YouTube selection.
                match self.try_download_via_spotify_match(&mut track).await {
                    Some(path) => Some(path),
                    None => {
                        tracing::warn!(
                            "No usable Spotify match — marking for manual YouTube selection"
                        );
                        if !track.needs_validation {
                            track.needs_validation = true;
                            track.validation_reason = Some("soundcloud_drm_protected".to_string());
                        }
                        None
                    }
                }
            }
            Err(e) => return Err(e),
        };

        if should_validate || file_path_opt.is_none() {
            tracing::warn!(
                "Track saved for manual validation — reason={:?}",
                track.validation_reason
            );
            let saved_track = self.save_track(conn, &track).await?;
            return Ok(saved_track);
        }

        let file_path = file_path_opt.expect("checked is_none above");

        // Step 3: Deduplication
        if existing_track.is_none() {
            tracing::info!("Deduping track in database");
            existing_track = self.dedupe_track(conn, &track).await;
        }

        match existing_track {
            Some(existing_track) => {
                tracing::info!(
                    "Existing track found in DB: {}, will compare quality",
                    existing_track.display()
                );

                let mut existing_track = existing_track;
                let new_track_is_better_quality = self
                    .track_service
                    .is_better_quality(&existing_track, &track);

                if new_track_is_better_quality {
                    tracing::warn!("New one has better quality, will replace");

                    // Merge nested metadata refs (album/artists) from the new track, then swap source/provider.
                    let mut track_for_merge = track.clone();
                    normalize_album_and_artist_refs_as_metadata(&mut track_for_merge);
                    existing_track.transpose_refs(&track_for_merge);
                    apply_source_provider_replacement(&mut existing_track, &track);

                    self.process_track_file(&mut existing_track, &file_path)
                        .await?;
                    let updated_track = self.save_track(conn, &existing_track).await?;
                    Ok(updated_track)
                } else {
                    tracing::warn!("New one has no better quality, skipping");

                    // Keep current audio source/provider, but keep Spotify (and downloader provider) as Metadata refs.
                    let mut track_for_merge = track.clone();
                    normalize_album_and_artist_refs_as_metadata(&mut track_for_merge);
                    demote_track_source_and_provider_to_metadata(&mut track_for_merge);
                    existing_track.transpose_refs(&track_for_merge);

                    let updated_track = self.save_track(conn, &existing_track).await?;
                    let _ = self.track_service.delete_track_file(&track)?;
                    Ok(updated_track)
                }
            }
            None => {
                tracing::info!("No existing track found in DB, processing new track");
                // Final Step: Tagging, moving and saving in DB
                self.process_track_file(&mut track, &file_path).await?;
                let inserted_track = self.save_track(conn, &track).await?;
                Ok(inserted_track)
            }
        }
    }

    /// Enrich metadata using metadata providers, and deduplicate entities in DB
    ///
    /// `for_ingest` — when `true`, uses `ingest_metadata_providers` from config
    /// (Spotify-first by default) instead of the standard download order.
    ///
    /// Returns:
    /// - boolean indicating if the track should be marked as "to validate"
    /// - boolean indicating if the track should be compared in quality (already exists in DB)
    async fn enrich_metada(
        &self,
        conn: &mut SqliteConnection,
        track: &mut Track,
        for_ingest: bool,
    ) -> SoundomeResult<(bool, Option<Track>)> {
        // Check if album/artists with same source ref url exist in DB and associate them
        let existing_album = track.album.as_ref().and_then(|a| {
            a.get_source()
                .or_else(|| a.get_metadata())
                .and_then(|s| s.external_url)
                .and_then(|url| self.album_service.get_by_url(conn, &url))
        });
        if let Some(existing_album) = existing_album {
            track.album = Some(existing_album);
        }

        for artist in &mut track.artists {
            if let Some(existing_artist) = artist
                .get_source()
                .or_else(|| artist.get_metadata())
                .and_then(|s| s.external_url)
                .and_then(|url| self.artist_service.get_by_url(conn, &url))
            {
                *artist = existing_artist;
            }
        }

        // Get metadata from all enabled providers
        let best_match = if for_ingest {
            tagger::enricher::get_best_match_from_track_for_ingest(track).await
        } else {
            tagger::enricher::get_best_match_from_track(track).await
        };

        // Apply best match metadata
        if let Match::Exact(matched_track) = best_match {
            // TODO: Check if ref already exists in DB, if yes then apply references recursively to track and unfound album/artists
            tracing::info!(
                "Exact match found from metadata provider: {:?}",
                matched_track.get_metadata().and_then(|m| m.external_url)
            );
            // find for existing tracks in the database

            if let Some(mb_ref) = matched_track
                .get_metadata()
                .and_then(|s| s.external_url.clone())
            {
                if let Some(existing_track) = self.track_service.get_by_url(conn, &mb_ref) {
                    tracing::warn!(
                        "Track already exists in DB with MusicBrainz ref: {}, skipping enrichment",
                        existing_track.display()
                    );
                    return Ok((false, Some(existing_track)));
                }
            }

            // Check if album/artists with same musicbrainz source url exist in DB and associate them
            let existing_album = track.album.as_ref().and_then(|a| {
                a.get_source()
                    .or_else(|| a.get_metadata())
                    .and_then(|s| s.external_url)
                    .and_then(|url| self.album_service.get_by_url(conn, &url))
            });
            if let Some(existing_album) = existing_album {
                track.album = Some(existing_album);
            }

            for artist in &mut track.artists {
                if let Some(existing_artist) = artist
                    .get_source()
                    .or_else(|| artist.get_metadata())
                    .and_then(|s| s.external_url)
                    .and_then(|url| self.artist_service.get_by_url(conn, &url))
                {
                    *artist = existing_artist;
                }
            }

            // Exact match: treat the enricher result as an authoritative, high-confidence
            // source so cleaner metadata (e.g. artist names) always replaces noisy source data.
            track.transpose_metadata_from_source(&matched_track);
            Ok((false, None)) // no need to validate
        } else if let Match::Partial(matched_track) = best_match {
            // Partial match: keep current (source) metadata, but attach MusicBrainz IDs/URLs for later review.
            // Do NOT transpose album data from partial match to avoid introducing incorrect album info.
            tracing::warn!(
                "Partial match found from metadata providers - will mark for validation"
            );

            track.transpose_refs_without_album(&matched_track);
            track.needs_validation = true;
            track.validation_reason = Some("metadata_partial_match".to_string());

            Ok((true, None))
        } else {
            // TODO: No match -> mark as "to validate"
            tracing::warn!("No match found from metadata providers");
            track.needs_validation = true;
            track.validation_reason = Some("metadata_no_match".to_string());
            Ok((true, None))
        }
    }

    /// Searches for the best download URL and downloads the track
    ///
    /// Returns the downloaded track with updated references and file_path
    /// Searches for the best download URL and downloads the track to the staging folder.
    /// The staging path is stored in `track.file_path`.
    async fn download_track(&self, track: &mut Track) -> SoundomeResult<PathBuf> {
        // Get the best download URL
        let provider_ref = downloader::search(track).await?;
        tracing::info!(
            "Found download URL from {:?}: {:?}",
            provider_ref.platform,
            provider_ref.external_url
        );
        track.references.push(provider_ref.clone());

        let staging_dir = PathBuf::from(&Config::get().general.temp_download_dir);

        // Download the track to staging
        let file_path = downloader::download(
            &track
                .get_source()
                .ok_or(Error::Custom("track source not defined".to_string()))?,
            &provider_ref,
            &track.title,
            staging_dir,
        )
        .await?;
        tracing::info!("Downloaded track to staging: {:?}", file_path);
        track.file_path = file_path.clone().into();

        Ok(file_path)
    }

    /// When a SoundCloud download fails as DRM-protected, check whether the track already
    /// carries a Spotify `Metadata` reference (typically attached during enrichment in
    /// `enrich_metada`, since Spotify is one of the tagger metadata providers). If so, retry
    /// via the existing Spotify → YouTube/YouTube Music matching flow (`downloader::search`'s
    /// `Platform::Spotify` branch) instead of immediately requiring manual YouTube selection.
    ///
    /// The track's `Source` reference is left untouched — SoundCloud is still where the user
    /// asked Soundome to import from. Only the resolved `Provider` reference and staged
    /// `file_path` are attached, and only on success.
    ///
    /// Returns `Some(path)` when the fallback download succeeded. Returns `None` when there is
    /// no Spotify metadata reference, or the fallback search/download itself failed, so the
    /// caller can fall back to the existing manual validation flow unchanged.
    async fn try_download_via_spotify_match(&self, track: &mut Track) -> Option<PathBuf> {
        let spotify_ref = track
            .references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Metadata && r.platform == Platform::Spotify)?
            .clone();

        tracing::info!(
            "DRM-protected SoundCloud track has a known Spotify reference ({:?}) — retrying via Spotify matching flow",
            spotify_ref.external_url
        );

        // Reuse `downloader::search`'s existing Spotify matching flow (YouTube Music, falling
        // back to YouTube) by presenting the Spotify reference as the `Source` on a throwaway
        // probe. `track`'s own `Source` reference (SoundCloud) is not modified.
        let mut probe = track.clone();
        probe
            .references
            .retain(|r| r.ref_type != ReferenceType::Source);
        probe.references.push(Reference {
            id: None,
            ref_type: ReferenceType::Source,
            platform: Platform::Spotify,
            external_id: spotify_ref.external_id,
            external_url: spotify_ref.external_url,
        });

        let provider_ref = match downloader::search(&probe).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Spotify-match fallback search found no candidate: {}", e);
                return None;
            }
        };
        tracing::info!(
            "Spotify-match fallback resolved a download URL from {:?}: {:?}",
            provider_ref.platform,
            provider_ref.external_url
        );

        // Download using the real (SoundCloud) source reference — `downloader::download`
        // already supports pairing a SoundCloud source with a YouTube/YouTube Music provider,
        // the same dispatch used by the manual DRM-fallback validation flow in
        // `finalize_validated_track`.
        let source_ref = track.get_source()?;
        let staging_dir = PathBuf::from(&Config::get().general.temp_download_dir);
        match downloader::download(&source_ref, &provider_ref, &track.title, staging_dir).await {
            Ok(path) => {
                track.file_path = Some(path.clone());
                track.references.push(provider_ref);
                Some(path)
            }
            Err(e) => {
                tracing::warn!("Spotify-match fallback download failed: {}", e);
                None
            }
        }
    }

    /// Simple deduplication based on comparition of title and artist(s) against existing tracks in DB
    async fn dedupe_track(&self, conn: &mut SqliteConnection, track: &Track) -> Option<Track> {
        let result = self
            .track_service
            .find_track_by_title_and_artist(conn, track);

        match result {
            Some(existing_track) => {
                tracing::warn!(
                    "Similar track found in DB: {}, will compare quality",
                    existing_track.display()
                );
                Some(existing_track)
            }
            None => None,
        }
    }

    /// Tag the downloaded file with the track metadata, then move it to the correct location
    async fn process_track_file(&self, track: &mut Track, file_path: &Path) -> SoundomeResult<()> {
        // Assign a SOUNDOME_ID if the track does not already have one.
        if track.soundome_id.is_none() {
            track.soundome_id = Some(Uuid::new_v4().to_string());
            tracing::debug!("Assigned SOUNDOME_ID: {:?}", track.soundome_id);
        }

        // Ensure file_path is set on the track — required by the organizer.
        // In the URL-download path this is already set by `download_track`; in the
        // local-ingest path the track comes from tag reading and has no path yet.
        if track.file_path.is_none() {
            track.file_path = Some(file_path.to_path_buf());
        }

        // Best-effort: download cover art from its URL and embed it in the file.
        let cover_url_opt = track.cover.clone();
        let cover_bytes: Option<Vec<u8>> = if let Some(url) = cover_url_opt {
            tokio::task::spawn_blocking(move || {
                reqwest::blocking::get(&url)
                    .and_then(|resp| resp.error_for_status())
                    .and_then(|resp| resp.bytes().map(|b| b.to_vec()))
                    .map_err(|e| {
                        tracing::warn!("Could not download cover art from {}: {}", url, e);
                        e
                    })
                    .ok()
            })
            .await
            .unwrap_or(None)
        } else {
            None
        };

        tagger::file::tag_file_with_track_and_cover(
            &file_path.to_path_buf(),
            track,
            cover_bytes.as_deref(),
        )?;
        tracing::info!("Tagged file with track metadata");

        // Move the file to the correct location
        let base_library_dir = Config::get().general.base_library_dir.clone();
        organizer::move_track_file(track, &base_library_dir)?;

        Ok(())
    }

    /// Re-tag and reorganize a track file if its metadata (especially artist or album) has changed.
    /// This is used when a user edits track metadata via the API.
    ///
    /// Returns true if the file was moved to a new location.
    pub async fn update_track_file_metadata(
        &self,
        old_track: &Track,
        new_track: &mut Track,
    ) -> SoundomeResult<bool> {
        // Check if the track has a file to update
        let mut file_path = match &old_track.file_path {
            Some(path) => path.clone(),
            None => {
                tracing::debug!("Track has no file, skipping file update");
                return Ok(false);
            }
        };

        // Resolve relative paths by joining with base_library_dir, but avoid duplication
        if !file_path.is_absolute() {
            let base_dir = PathBuf::from(&Config::get().general.base_library_dir);
            let base_dir_str = base_dir.to_string_lossy();
            let file_path_str = file_path.to_string_lossy();

            // Only join if the file_path doesn't already start with the base_dir
            if !file_path_str.starts_with(base_dir_str.as_ref()) {
                file_path = base_dir.join(&file_path);
            }
            tracing::debug!("Resolved path to: {:?}", file_path);
        }

        // Check if file still exists
        if !file_path.exists() {
            tracing::warn!(
                "Track file does not exist at {:?}, skipping file update",
                file_path
            );
            return Ok(false);
        }

        // Check if artist, album, or title metadata has changed
        let artist_names_changed = old_track
            .artists
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            != new_track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>();

        let album_changed = old_track.album.as_ref().map(|a| a.title.clone())
            != new_track.album.as_ref().map(|a| a.title.clone());

        let title_changed = old_track.title != new_track.title;

        let location_changed = artist_names_changed || album_changed || title_changed;

        // Re-tag the file with new metadata
        tracing::info!("Re-tagging file with updated metadata");
        tagger::file::tag_file_with_track(&file_path, new_track)?;

        // If any location-affecting metadata changed (artist, album, or title), reorganize the file.
        // This is safer than renaming directly because the organizer handles path normalization.
        if location_changed {
            tracing::info!(
                "Metadata changed (artist={}, album={}, title={}), reorganizing file",
                artist_names_changed,
                album_changed,
                title_changed
            );
            // Update the file_path in new_track to the resolved absolute path before organizing
            new_track.file_path = Some(file_path);
            let base_library_dir = Config::get().general.base_library_dir.clone();
            organizer::move_track_file(new_track, &base_library_dir)?;

            // Normalize the file_path back to relative for storage in DB
            // The file_path is now absolute, so make it relative to base_library_dir
            if let Some(abs_path) = &new_track.file_path {
                let base_path = std::path::PathBuf::from(&base_library_dir);
                if let Ok(rel_path) = abs_path.strip_prefix(&base_path) {
                    let rel_path_str = format!("./{}", rel_path.to_string_lossy());
                    new_track.file_path = Some(std::path::PathBuf::from(rel_path_str));
                    tracing::debug!("Normalized path to relative: {:?}", new_track.file_path);
                }
            }
            Ok(true)
        } else {
            tracing::debug!("File location unchanged, no reorganization needed");
            Ok(false)
        }
    }

    /// Save the track in the database
    async fn save_track(
        &self,
        conn: &mut SqliteConnection,
        track: &Track,
    ) -> SoundomeResult<Track> {
        let inserted_track = self.track_service.create_or_update(conn, track)?;
        tracing::info!("Saved track in the database");
        Ok(inserted_track)
    }

    /// Best-effort M3U8 export: fetch playlist tracks from DB and write the file.
    /// Failures are logged as warnings and do not propagate.
    fn export_playlist_m3u8(
        &self,
        conn: &mut SqliteConnection,
        playlist: &Playlist,
        playlist_id: i32,
    ) {
        match self.playlist_service.export_m3u8(conn, playlist_id) {
            Ok(path) => tracing::info!("M3U8 playlist exported: {:?}", path),
            Err(e) => tracing::warn!(
                "M3U8 export failed for playlist \"{}\": {}",
                playlist.name,
                e
            ),
        }
    }
}

fn normalize_album_and_artist_refs_as_metadata(track: &mut Track) {
    if let Some(album) = &mut track.album {
        for r in &mut album.references {
            r.ref_type = ReferenceType::Metadata;
            r.id = None;
        }
        for artist in &mut album.artists {
            for r in &mut artist.references {
                r.ref_type = ReferenceType::Metadata;
                r.id = None;
            }
        }
    }

    for artist in &mut track.artists {
        for r in &mut artist.references {
            r.ref_type = ReferenceType::Metadata;
            r.id = None;
        }
    }
}

fn demote_track_source_and_provider_to_metadata(track: &mut Track) {
    for r in &mut track.references {
        if r.ref_type == ReferenceType::Source || r.ref_type == ReferenceType::Provider {
            r.ref_type = ReferenceType::Metadata;
            r.id = None;
        }
    }
}

fn same_ref_identity(a: &shared::models::Reference, b: &shared::models::Reference) -> bool {
    a.platform == b.platform && a.external_id == b.external_id && a.external_url == b.external_url
}

/// Try to extract a track number from a file name when the embedded tag is absent.
///
/// Recognises common patterns:
///   "08 - Title.flac"   → 8
///   "08_Title.flac"     → 8
///   "08. Title.flac"    → 8
///   "08Title.flac"      → 8  (leading digits only)
///   "Track08.flac"      → ignored (no leading digits)
fn infer_track_number_from_filename(path: &Path) -> Option<i32> {
    let stem = path.file_stem()?.to_string_lossy();
    // Match 1–3 leading digits optionally followed by a separator character.
    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || digits.len() > 3 {
        return None;
    }
    digits
        .parse::<i32>()
        .ok()
        .filter(|&n| (1..=999).contains(&n))
}

fn apply_source_provider_replacement(existing_track: &mut Track, new_track: &Track) {
    let new_source = new_track.get_source();
    let new_provider = new_track.get_provider();

    // If we cannot determine both, do nothing (better to keep existing state).
    let (Some(new_source), Some(new_provider)) = (new_source, new_provider) else {
        return;
    };

    let old_source = existing_track.get_source();
    let old_provider = existing_track.get_provider();

    // Remove all existing Source/Provider refs; we'll re-add exactly one of each.
    existing_track
        .references
        .retain(|r| r.ref_type != ReferenceType::Source && r.ref_type != ReferenceType::Provider);

    let mut new_source = new_source;
    new_source.id = None;
    new_source.ref_type = ReferenceType::Source;
    let mut new_provider = new_provider;
    new_provider.id = None;
    new_provider.ref_type = ReferenceType::Provider;

    existing_track.references.push(new_source.clone());
    existing_track.references.push(new_provider.clone());

    // Demote old source/provider as metadata (dedupe if they were identical).
    let mut candidates: Vec<shared::models::Reference> = Vec::new();
    if let Some(old_source) = old_source {
        if !same_ref_identity(&old_source, &new_source) {
            let mut r = old_source;
            r.id = None;
            r.ref_type = ReferenceType::Metadata;
            candidates.push(r);
        }
    }
    if let Some(old_provider) = old_provider {
        if !same_ref_identity(&old_provider, &new_provider) {
            let mut r = old_provider;
            r.id = None;
            r.ref_type = ReferenceType::Metadata;
            candidates.push(r);
        }
    }

    for candidate in candidates {
        let already = existing_track.references.iter().any(|r| {
            r.ref_type == candidate.ref_type
                && r.platform == candidate.platform
                && r.external_id == candidate.external_id
                && r.external_url == candidate.external_url
        });
        if !already {
            existing_track.references.push(candidate);
        }
    }
}
