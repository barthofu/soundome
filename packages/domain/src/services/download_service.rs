use std::{path::PathBuf, sync::Arc};

use config::Config;
use diesel::SqliteConnection;
use fetcher::{Fetcher, Source};
use shared::{errors::Error, models::Track, types::SoundomeResult, utils::enums::Match};
use shared::models::ReferenceType;
use tagger::TagProvider;

use super::{album_service::AlbumService, artist_service::ArtistService, track_service::TrackService};

pub struct DownloadService {
    track_service: Arc<TrackService>,
    album_service: Arc<AlbumService>,
    artist_service: Arc<ArtistService>,
}

// TODO: gestion de la playlist
// TODO: manage "to validate" tracks
impl DownloadService {
    pub fn new(
        track_service: Arc<TrackService>,
        album_service: Arc<AlbumService>,
        artist_service: Arc<ArtistService>,
    ) -> Self {
        Self { 
            track_service,
            album_service,
            artist_service,
        }
    }

    /// Main entry point for downloading a track from a given URL (from any supported platform)
    pub async fn download_track_from_url(&self, url: &str, conn: &mut SqliteConnection) -> SoundomeResult<Track> {
        tracing::info!("===========\nDownloading track from {:?}\n------", url);

        // Check if track already exists in DB
        if let Some(t) = self.track_service.get_by_url(conn, url) {
            return Err(Error::TrackExists(t.display()));
        }

        let fetcher = Fetcher::new().await;

        // Fetch track info from URL
        let mut track = fetcher.get_track_from_url(url).await?;
        fetcher.clean_track_metadata(&mut track).await?;
        tracing::info!("Fetched track info from {}: {}", track.get_source_platform().as_ref(), track.display());
    
        // Orchestrator workflow
        let final_track = self.orchestrator_workflow(conn, track).await?;
        Ok(final_track)
    } 

    /// Main entry point for downloading a playlist from a given URL (from any supported platform)
    pub async fn sync_playlist_from_url(&self, url: &str, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>> {
        tracing::info!("====================\nDownloading playlist from {:?}\n---------", url);

        let fetcher = Fetcher::new().await;
        let playlist_tracks = fetcher.get_playlist_tracks_from_url(url).await?;
        let total_tracks = playlist_tracks.len();
        tracing::info!("Found {} tracks in playlist", total_tracks);

        // Filter out existing tracks and collect new ones
        let mut new_tracks: Vec<Track> = Vec::new();
        let mut existing_count = 0;
        for pt in playlist_tracks {
            let track = &pt.track;
            let url = track.get_source()
                .and_then(|s| s.external_url.clone())
                .unwrap_or_else(|| "unknown".to_string());
            if self.track_service.get_by_url(conn, &url).is_some() {
                tracing::warn!("   -> Track already exists in DB, skipping: {}", track.display());
                existing_count += 1;
            } else {
                new_tracks.push(track.clone());
            }
        }

        tracing::info!("{} new tracks to download after filtering existing ones", new_tracks.len());

        // Clean metadata for all new tracks
        if let Err(e) = fetcher.clean_tracks_metadata(&mut new_tracks.iter_mut().collect::<Vec<_>>()).await {
            tracing::info!("Failed to clean tracks title and artist name: {}", e);
        }

        // Process each new track
        let mut new_processed_tracks = Vec::new();
        for track in &new_tracks {
            tracing::info!("Processing track: {}", track.display());
            match self.orchestrator_workflow(conn, track.clone()).await {
                Ok(t) => {
                    tracing::info!("Successfully processed track: {}", t.display());
                    new_processed_tracks.push(t);
                },
                Err(e) => tracing::error!("Error downloading track {}: {:?}", track.display(), e)
            }
        }

        tracing::info!(
            "Downloaded {}/{} tracks from playlist, {} existing tracks and {} errors",
            new_processed_tracks.len(),
            total_tracks,
            existing_count,
            new_tracks.len() - new_processed_tracks.len(),
        );

        Ok(new_processed_tracks)
    }

    // ============================================================================================
    // == Sub private and utils methods
    // ============================================================================================

    async fn orchestrator_workflow(&self, conn: &mut SqliteConnection, track: Track) -> SoundomeResult<Track> {
        let mut track = track;

        // Step 1: Enrich metadata
        tracing::info!("Getting metadata via MusicBrainz");
        let (should_validate, mut existing_track) = self.enrich_metada(conn, &mut track).await?;

        // TODO: temporary, see Class level todo comment
        if should_validate {
            tracing::warn!("Stopping workflow - marked for validation");
            track.needs_validation = true;
            let saved_track = self.save_track(conn, &track).await?;
            return Ok(saved_track);
        }

        // Step 2: Download
        tracing::info!("Searching and downloading track from provider");
        let file_path = self.download_track(&mut track).await?;

        // Step 3: Deduplication
        if existing_track.is_none() {
            tracing::info!("Deduping track in database");
            existing_track = self.dedupe_track(conn, &track).await;
        }

        match existing_track {
            Some(existing_track) => {
                tracing::info!("Existing track found in DB: {}, will compare quality", existing_track.display());
                
                let mut existing_track = existing_track;
                let new_track_is_better_quality = self.track_service.is_better_quality(&existing_track, &track);

                if new_track_is_better_quality {
                    tracing::warn!("New one has better quality, will replace");

                    // Merge nested metadata refs (album/artists) from the new track, then swap source/provider.
                    let mut track_for_merge = track.clone();
                    normalize_album_and_artist_refs_as_metadata(&mut track_for_merge);
                    existing_track.transpose_refs(&track_for_merge);
                    apply_source_provider_replacement(&mut existing_track, &track);

                    self.process_track_file(&mut existing_track, &file_path).await?;
                    let updated_track = self.save_track(conn, &existing_track).await?;
                    return Ok(updated_track);
                } else {
                    tracing::warn!("New one has no better quality, skipping");

                    // Keep current audio source/provider, but keep Spotify (and downloader provider) as Metadata refs.
                    let mut track_for_merge = track.clone();
                    normalize_album_and_artist_refs_as_metadata(&mut track_for_merge);
                    demote_track_source_and_provider_to_metadata(&mut track_for_merge);
                    existing_track.transpose_refs(&track_for_merge);

                    let updated_track = self.save_track(conn, &existing_track).await?;
                    let _ = self.track_service.delete_track_file(&track)?;
                    return Ok(updated_track);
                }
            }
            None => {
                tracing::info!("No existing track found in DB, processing new track");
                // Final Step: Tagging, moving and saving in DB
                self.process_track_file(&mut track, &file_path).await?;
                let inserted_track = self.save_track(conn, &track).await?;
                return Ok(inserted_track);
            }
        }
    }

    /// Enrich metadata using metadata providers, and deduplicate entities in DB
    /// 
    /// Returns:
    /// - boolean indicating if the track should be marked as "to validate"
    /// - boolean indicating if the track should be compared in quality (already exists in DB)
    async fn enrich_metada(&self, conn: &mut SqliteConnection, track: &mut Track) -> SoundomeResult<(bool, Option<Track>)> {
        // Check if album/artists with same source ref url exist in DB and associate them
        let existing_album = track
            .album
            .as_ref()
            .and_then(|a|
                a.get_source()
                    .or_else(|| a.get_metadata())
                    .and_then(|s| s.external_url)
                    .and_then(|url| self.album_service.get_by_url(conn, &url))
            );
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

        // Get MusicBrainz metadata
        let musicbrainz = tagger::providers::musicbrainz::MusicBrainz::new();
        let best_match = musicbrainz
            .get_best_match_from_track(&track)
            .await;

        // Apply best match metadata
        if let Match::Exact(matched_track) = best_match {
            // TODO: Check if MusicBrainz ref already exists in DB, if yes then apply references recursively to track and unfound album/artists
            tracing::info!("Exact match found from MusicBrainz: {:?}", matched_track.get_metadata().and_then(|m| m.external_url));
            // find for existing tracks in the database 

            if let Some(mb_ref) = matched_track.get_metadata().and_then(|s| s.external_url.clone()) {
                if let Some(existing_track) = self.track_service.get_by_url(conn, &mb_ref) {
                    tracing::warn!("Track already exists in DB with MusicBrainz ref: {}, skipping enrichment", existing_track.display());
                    return Ok((false, Some(existing_track)));
                }
            }

            // Check if album/artists with same musicbrainz source url exist in DB and associate them
            let existing_album = track
                .album
                .as_ref()
                .and_then(|a|
                    a.get_source()
                        .or_else(|| a.get_metadata())
                        .and_then(|s| s.external_url)
                        .and_then(|url| self.album_service.get_by_url(conn, &url))
                );
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

            track.transpose_metadata(&matched_track);
            Ok((false, None)) // no need to validate
        } else if let Match::Partial(matched_track) = best_match {
            // Partial match: keep current (source) metadata, but attach MusicBrainz IDs/URLs for later review.
            tracing::warn!("Partial match found from MusicBrainz - will mark for validation");

            track.transpose_refs(&matched_track);
            track.needs_validation = true;
            track.validation_reason = Some("musicbrainz_partial_match".to_string());

            Ok((true, None))
        } else {
            // TODO: No match -> mark as "to validate"
            tracing::warn!("No match found from MusicBrainz");
            track.needs_validation = true;
            track.validation_reason = Some("musicbrainz_no_match".to_string());
            Ok((true, None))
        }
    }

    /// Searches for the best download URL and downloads the track
    /// 
    /// Returns the downloaded track with updated references and file_path
    async fn download_track(&self, track: &mut Track) -> SoundomeResult<PathBuf> {
        // Get the best download URL
        let provider_ref = downloader::search(&track).await?;
        tracing::info!("Found download URL from {:?}: {:?}", provider_ref.platform, provider_ref.external_url);
        track.references.push(provider_ref.clone());

        // Download the track
        let file_path = downloader::download(
            &track.get_source().ok_or(Error::Custom("track source not defined".to_string()))?,
            &provider_ref,
            &track.title,
        )
        .await?;
        // let file_path = PathBuf::from("/home/coder/soundome/library/Générations.mp3");
        tracing::info!("Downloaded track to {:?}", file_path);
        track.file_path = file_path.clone().into();

        Ok(file_path)
    }

    /// Simple deduplication based on comparition of title and artist(s) against existing tracks in DB
    async fn dedupe_track(&self, conn: &mut SqliteConnection, track: &Track) -> Option<Track> {
        let result = self.track_service.find_track_by_title_and_artist(conn, track);

        match result {
            Some(existing_track) => {
                tracing::warn!("Similar track found in DB: {}, will compare quality", existing_track.display());
                Some(existing_track)
            },
            None => {
                None
            },
        }
    }

    /// Tag the downloaded file with the track metadata, then move it to the correct location
    async fn process_track_file(&self, track: &mut Track, file_path: &PathBuf) -> SoundomeResult<()> {
        tagger::file::tag_file_with_track(&file_path.clone(), &track)?;
        tracing::info!("Tagged file with downloaded_track metadata");

        // Move the file to the correct location
        let base_library_dir = Config::get().general.base_library_dir.clone();
        organizer::move_track_file(track, &base_library_dir)?;

        Ok(())
    }

    /// Save the track in the database
    async fn save_track(&self, conn: &mut SqliteConnection, track: &Track) -> SoundomeResult<Track> {
        let inserted_track = self.track_service.create_or_update(conn, track)?;
        tracing::info!("Saved track in the database");
        Ok(inserted_track)
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
