use config::model::AppConfig;
// On suppose que ces modules existent dans le monorepo et exposent les fonctions nécessaires.
use downloader;
use fetcher;
use organizer::move_track_file;
use shared::{
    errors::Error, models::{Platform, Track}, types::SoundomeResult, utils::enums::Match
};
use tagger::TagProvider;

pub struct Orchestrator {
    config: AppConfig,
}

impl Orchestrator {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub async fn download_track_from_url(&self, url: &str) -> Result<Track, Error> {
        println!("===========\nDownloading track from {:?}\n------", url);
        // Fetch track metadata
        let mut track = fetcher::get_track_from_url(url, &self.config).await?;
        println!(
            "Fetched track from {}: {}",
            track.get_source()
                .map(|s| s.platform)
                .unwrap_or(Platform::Unknown)
                .as_ref(),
            track.display()
        );

        // Download the track
        track = self.download_track(track).await?;
        Ok(track)
    }

    pub async fn download_playlist_from_url(&self, url: &str) -> SoundomeResult<Vec<Track>> {
        println!(
            "====================\nDownloading playlist from {:?}\n---------",
            url
        );
        // Fetch playlist metadata
        let playlist_items = fetcher::get_playlist_tracks_from_url(url, &self.config).await?;
        println!("Found {} tracks in playlist", playlist_items.len());

        let mut tracks = vec![];
        let mut error_count = 0;
        for playlist_item in playlist_items {
            let title = playlist_item.track.display();
            println!("===========\nDownloading track {}", title);
            let track = self.download_track(playlist_item.track).await;
            match track {
                Ok(t) => {
                    println!("Downloaded track from playlist: {}", &t.display());
                    tracks.push(t);
                }
                Err(e) => {
                    error_count += 1;
                    eprintln!("/!\\ Error downloading track {}: {}", title, e.to_string());
                }
            }
        }

        println!(
            "Downloaded {} tracks from playlist, {} errors",
            tracks.len() - error_count,
            error_count
        );

        Ok(tracks)
    }

    async fn download_track(&self, track: Track) -> SoundomeResult<Track> {
        let mut downloaded_track = track;

        // Get the best download URL
        let provider_ref = downloader::search(&downloaded_track, &self.config).await?;
        println!("Found download URL from {:?}: {:?}", provider_ref.platform, provider_ref.external_url);
        downloaded_track.references.push(provider_ref);

        // Download the track
        let file_path = downloader::download(
            &downloaded_track,
            &self.config,
        )
        .await?;
        println!("Downloaded track to {:?}", file_path);
        downloaded_track.file_path = file_path.clone().into();

        // Get MusicBrainz metadata
        let musicbrainz = tagger::providers::musicbrainz::MusicBrainz::new();
        let best_match = musicbrainz
            .get_best_match_from_track(&downloaded_track)
            .await;
        if let Match::Exact(matched_track) = best_match {
            println!("Exact match found from MusicBrainz");
            downloaded_track.transpose_metadata(&matched_track);
        } else if let Match::Partial(_) = best_match {
            // TODO: Handle partial match
            println!("Partial match found from MusicBrainz");
        } else {
            println!("No match found from MusicBrainz");
        }

        tagger::file::tag_file_with_track(&file_path.clone(), &downloaded_track)?;
        println!("Tagged file with downloaded_track metadata");

        // Move the file to the correct location
        move_track_file(&mut downloaded_track, &self.config.general.base_dir)?;

        // Save in the database
        let mut conn = database::get_connection(&self.config.database.url);
        database::services::track::create_track(&mut conn, &downloaded_track).unwrap(); // TODO: tmp
        println!("Saved track in the database");

        Ok(downloaded_track)
    }
}
