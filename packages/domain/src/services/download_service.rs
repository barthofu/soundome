use std::{path::PathBuf, sync::Arc};

use config::model::AppConfig;
use diesel::SqliteConnection;
use shared::{errors::Error, models::Track, types::SoundomeResult, utils::enums::Match};
use tagger::TagProvider;
use tracing::{error, info, warn};

use super::{album_service::AlbumService, artist_service::ArtistService, track_service::TrackService};

pub struct DownloadService {
    track_service: Arc<TrackService>,
    album_service: Arc<AlbumService>,
    artist_service: Arc<ArtistService>,
    config: Arc<AppConfig>
}

// TODO: gestion de la playlist
// TODO: manage "to validate" tracks
impl DownloadService {
    pub fn new(
        track_service: Arc<TrackService>,
        album_service: Arc<AlbumService>,
        artist_service: Arc<ArtistService>,
        config: Arc<AppConfig>,
    ) -> Self {
        Self { 
            track_service,
            album_service,
            artist_service,
            config
        }
    }

    pub async fn download_track_from_url(&self, url: &str, conn: &mut SqliteConnection) -> SoundomeResult<Track> {
        info!("===========\nDownloading track from {:?}\n------", url);

        // Check if track already exists in DB
        if let Some(t) = self.track_service.get_by_url(conn, url) {
            return Err(Error::TrackExists(t.display()));
        }

        // Fetch track info from URL
        let track = fetcher::get_track_from_url(url, &self.config).await?;
        info!("Fetched track info from {}: {}", track.get_source_platform().as_ref(), track.display());
    
        // Orchestrator workflow
        let final_track = self.orchestrator_workflow(conn, track).await?;
        Ok(final_track)
    } 

    pub async fn download_playlist_from_url(&self, url: &str, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>> {
        info!(
            "====================\nDownloading playlist from {:?}\n---------",
            url
        );
        
        // Fetch tracks from playlist
        let playlist_tracks = fetcher::get_playlist_tracks_from_url(url, &self.config).await?;
        info!("Found {} tracks in playlist", playlist_tracks.len());

        let mut new_tracks = Vec::new();
        let mut existing_tracks = Vec::new();
        for playlist_track in &playlist_tracks {
            let track = &playlist_track.track;
            let url = track.get_source()
                .and_then(|s| s.external_url.clone())
                .unwrap_or_else(|| "unknown".to_string());
            info!(" - {} ({})", track.display(), url);

            // Check if track already exists in DB
            if let Some(t) = self.track_service.get_by_url(conn, &url) {
                warn!("   -> Track already exists in DB, skipping download");
                existing_tracks.push(t);
                continue;
            }

            // Orchestrator workflow
            match self.orchestrator_workflow(conn, track.clone()).await {
                Ok(t) => new_tracks.push(t),
                Err(e) => {
                    error!("Error downloading track {}: {:?}", track.display(), e);
                }
            }
        }

        info!(
            "Downloaded {}/{} tracks from playlist, {} existing tracks and {} errors",
            new_tracks.len(),
            playlist_tracks.len(),
            existing_tracks.len(),
            playlist_tracks.len() - (new_tracks.len() + existing_tracks.len())
        );

        Ok(new_tracks)
    }

    // ============================================================================================
    // == Sub private and utils methods
    // ============================================================================================

    async fn orchestrator_workflow(&self, conn: &mut SqliteConnection, mut track: Track) -> SoundomeResult<Track> {
        // Step 2: Enrichissement des métadonnées via MusicBrainz
        info!("Getting metadata via MusicBrainz");
        let (enriched_track, should_continue_download) = self.enrich_metada(conn, track).await?;

        // TODO: temporary, see Class level todo comment
        if !should_continue_download {
            warn!("Stopping workflow - marked for validation");
            return Ok(enriched_track);
        }

        // Step 3: Téléchargement
        info!("Searching and downloading track from provider");
        let (downloaded_track, file_path) = self.download_track(enriched_track).await?;

        // Step 4: Déduplication
        info!("Deduping track in database");
        let existing_track = self.dedupe_track(conn, &downloaded_track).await;

        // Step 5: Quality comparison and deduplication if existing
        // track = self.compare_quality_and_dedupe(&existing_track, &track).await?;

        // Final Step: Tagging, moving and saving in DB
        info!("Tagging, moving and saving track in database");
        track = self.process_track(conn, downloaded_track, &file_path).await?;

        Ok(track)
    }

    /// Enrich metadata using metadata providers, and deduplicate entities in DB
    async fn enrich_metada(&self, conn: &mut SqliteConnection, track: Track) -> SoundomeResult<(Track, bool)> {
        let mut enriched_track = track;

        // Check if album/artists exist in DB and associate them
        let existing_album = enriched_track
            .album
            .and_then(|a|
                a.get_source().and_then(|s| s.external_url).and_then(|url| {
                    self.album_service.get_by_url(conn, &url)
                }
            ));
        enriched_track.album = existing_album;

        for artist in &mut enriched_track.artists {
            if let Some(existing_artist) = artist.get_source().and_then(|s| s.external_url).and_then(|url| {
                self.artist_service.get_by_url(conn, &url)
            }) {
                *artist = existing_artist;
            }
        }

        // Get MusicBrainz metadata
        let musicbrainz = tagger::providers::musicbrainz::MusicBrainz::new();
        let best_match = musicbrainz
            .get_best_match_from_track(&enriched_track)
            .await;

        // Apply best match metadata
        let should_validate = if let Match::Exact(matched_track) = best_match {
            // TODO: Check if MusicBrainz ref already exists in DB, if yes then apply references recursively to track and unfound album/artists
            info!("Exact match found from MusicBrainz");
            enriched_track.transpose_metadata(&matched_track);
            false // no need to validate
        } else if let Match::Partial(_) = best_match {
            // TODO: Handle partial match -> no transpose, but associate MusicBrainz ref and mark as "to validate"
            warn!("Partial match found from MusicBrainz");
            true // should validate
        } else {
            // TODO: No match -> mark as "to validate"
            warn!("No match found from MusicBrainz");
            true // should validate
        };

        Ok((enriched_track, should_validate))
    }

    async fn download_track(&self, track: Track) -> SoundomeResult<(Track, PathBuf)> {
        let mut downloaded_track = track;

        // Get the best download URL
        let provider_ref = downloader::search(&downloaded_track, &self.config).await?;
        info!("Found download URL from {:?}: {:?}", provider_ref.platform, provider_ref.external_url);
        downloaded_track.references.push(provider_ref);

        // Download the track
        let file_path = downloader::download(
            &downloaded_track,
            &self.config,
        )
        .await?;
        info!("Downloaded track to {:?}", file_path);
        downloaded_track.file_path = file_path.clone().into();

        Ok((downloaded_track, file_path))
    }

    /// Simple deduplication based on comparition of title and artist(s) against existing tracks in DB
    async fn dedupe_track(&self, conn: &mut SqliteConnection, track: &Track) -> Option<Track> {
        let result = self.track_service.find_track_by_title_and_artist(conn, track);

        match result {
            Some(existing_track) => {
                warn!("Similar track found in DB: {}, will compare quality", existing_track.display());
                Some(existing_track)
            },
            None => {
                info!("No similar track found in DB, proceeding with download");
                None
            },
        }
    }

    async fn compare_quality_and_dedupe(&self, track: &Track) -> SoundomeResult<()> {
        // TODO: if found, dedupe based on quality
        todo!()
    }

    async fn process_track(&self, _conn: &mut SqliteConnection, track: Track, file_path: &PathBuf) -> SoundomeResult<Track> {
        let mut track = track;

        tagger::file::tag_file_with_track(&file_path.clone(), &track)?;
        info!("Tagged file with downloaded_track metadata");

        // Move the file to the correct location
        organizer::move_track_file(&mut track, &self.config.general.base_library_dir)?;

        // Save in the database
        // let mut conn = database::get_connection(&self.config.database.url);
        // database::services::track::create_track(&mut conn, &downloaded_track).unwrap(); // TODO: tmp
        info!("Saved track in the database");

        Ok(track)
    }
}
