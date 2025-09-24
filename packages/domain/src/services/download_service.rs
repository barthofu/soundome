use std::{path::PathBuf, sync::Arc};

use config::Config;
use diesel::SqliteConnection;
use fetcher::{Fetcher, Source};
use shared::{errors::Error, models::Track, types::SoundomeResult, utils::enums::Match};
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

        // Step 1: Enrichissement des métadonnées via MusicBrainz
        tracing::info!("Getting metadata via MusicBrainz");
        let (should_validate, mut existing_track) = self.enrich_metada(conn, &mut track).await?;

        // TODO: temporary, see Class level todo comment
        if should_validate {
            tracing::warn!("Stopping workflow - marked for validation");
            return Ok(track);
        }

        // // Step 2: Déduplication
        // tracing::info!("Deduping track in database");
        // let existing_track = self.dedupe_track(conn, &enriched_track).await;

        // if let Some(existing_track) = existing_track {
        //     tracing::warn!("Track already exists in DB: {}, skipping download", existing_track.display());
        //     return Ok(existing_track);
        // }

        // Ok(enriched_track)

        // Step 2: Téléchargement
        tracing::info!("Searching and downloading track from provider");
        let file_path = self.download_track(&mut track).await?;

        // Step 3: Déduplication
        if existing_track.is_none() {
            tracing::info!("Deduping track in database");
            existing_track = self.dedupe_track(conn, &track).await;
        }

        match existing_track {
            Some(existing_track) => {
                tracing::info!("Existing track found in DB: {}, will compare quality", existing_track.display());

                // existing_track.transpose_refs(track);

                let new_track_is_better_quality = self.track_service.is_better_quality(&existing_track, &track);

                if new_track_is_better_quality {
                    tracing::warn!("New one has better quality, will replace");
                    self.process_track(conn, existing_track, &file_path).await?;
                    return Ok(existing_track);
                } else {
                    tracing::warn!("New one has no better quality, skipping");
                    return Ok(existing_track.clone());
                }
            }
            None => {

            }
        }
        

        // l
        // if let Some(existing_track) = existing_track {
        //     if new_track_is_better_quality {
        //         tracing::warn!("A similar track exists in DB: {}, but the new one has better quality, will replace", existing_track.display());
        //     } else {
        //         tracing::warn!("A similar track exists in DB: {}, skipping download", existing_track.display());
        //         return Ok(existing_track);
        //     }
        // }

        // Final Step: Tagging, moving and saving in DB
        tracing::info!("Tagging, moving and saving track in database");
        track = self.process_track(conn, track, &file_path).await?;

        Ok(track)
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
                a.get_source().and_then(|s| s.external_url).and_then(|url| {
                    self.album_service.get_by_url(conn, &url)
                }
            ));
        track.album = existing_album;

        for artist in &mut track.artists {
            if let Some(existing_artist) = artist.get_source().and_then(|s| s.external_url).and_then(|url| {
                self.artist_service.get_by_url(conn, &url)
            }) {
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

            if let Some(mb_ref) = matched_track.get_source().and_then(|s| s.external_url.clone()) {
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
                    a.get_metadata().and_then(|s| s.external_url).and_then(|url| {
                        self.album_service.get_by_url(conn, &url)
                    }
                ));
            track.album = existing_album;

            for artist in &mut track.artists {
                if let Some(existing_artist) = artist.get_metadata().and_then(|s| s.external_url).and_then(|url| {
                    self.artist_service.get_by_url(conn, &url)
                }) {
                    *artist = existing_artist;
                }
            }

            track.transpose_metadata(&matched_track);
            Ok((false, None)) // no need to validate
        } else if let Match::Partial(_) = best_match {
            // TODO: Handle partial match -> no transpose, but associate MusicBrainz ref and mark as "to validate"
            tracing::warn!("Partial match found from MusicBrainz");
            Ok((true, None)) // should validate
        } else {
            // TODO: No match -> mark as "to validate"
            tracing::warn!("No match found from MusicBrainz");
            // TODO: change to true
            Ok((true, None)) // should validate
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

    async fn process_track(&self, conn: &mut SqliteConnection, track: Track, file_path: &PathBuf) -> SoundomeResult<Track> {
        let mut track = track;

        tagger::file::tag_file_with_track(&file_path.clone(), &track)?;
        tracing::info!("Tagged file with downloaded_track metadata");

        // Move the file to the correct location
        let base_library_dir = Config::get().general.base_library_dir.clone();
        organizer::move_track_file(&mut track, &base_library_dir)?;

        // Save in the database
        self.track_service.create_or_ignore(conn, &track)?; 
        tracing::info!("Saved track in the database");

        Ok(track)
    }
}
