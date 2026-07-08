mod mappers;

use async_trait::async_trait;
use config::Config;
use fancy_regex::Regex;
use futures::future::join_all;
use mappers::convert_track;
use rustypipe::{
    client::RustyPipe,
    model::{MusicArtist, TrackItem},
};
use shared::{
    errors::Error,
    http::HttpClientBuilder,
    models::{Album, Artist, Platform, Playlist, PlaylistTrack, Track},
    types::SoundomeResult,
};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::Source;

pub struct YoutubeMusic {
    client: RustyPipe,
}

impl YoutubeMusic {
    const TRACK_REGEX: &str = r#"/(?:https?:)?(?:\/\/)?(?:[0-9A-Z-]+\.)?(?:youtu\.be\/|youtube(?:-nocookie)?\.com\S*?[^\w\s-])([\w-]{11})(?=[^\w-]|$)(?![?=&+%\w.-]*(?:['"][^<>]*>|<\/a>))[?=&+%\w.-]*/gim"#;
    const PLAYLIST_REGEX: &str =
        r"^https://music\.youtube\.com/(?:playlist|watch)\?.*\blist=([^#&?]+)";
    const ARTIST_REGEX: &str = r"^https:\/\/music\.youtube\.com\/channel\/([A-Za-z0-9_-]+)$";

    /// Caps how many tracks are enriched (per-track artist/album lookups) at the
    /// same time. Firing every track at once (previously via `join_all`) meant
    /// hundreds of simultaneous requests for large playlists, which is enough to
    /// make YouTube Music rate-limit/return malformed responses (surfaced as
    /// rustypipe retry warnings, error reports, and even a panic in rustypipe's
    /// internal visitor-data refresh under load).
    const MAX_CONCURRENT_ENRICHMENT: usize = 5;

    /// Helper to apply concurrency limits to a batch of track enrichment futures.
    /// Uses a Semaphore to limit concurrent requests while preserving order.
    async fn enrich_tracks_with_limit(
        &self,
        tracks: impl Iterator<Item = TrackItem>,
    ) -> Vec<Track> {
        let semaphore = Arc::new(Semaphore::new(Self::MAX_CONCURRENT_ENRICHMENT));
        let futures = tracks.map(|track| {
            let semaphore = Arc::clone(&semaphore);
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                self.get_complete_track_from_music_track(track).await
            }
        });
        join_all(futures).await
    }

    pub fn new() -> SoundomeResult<Self> {
        let client = match Config::get().proxy.as_ref() {
            Some(proxy_config) if proxy_config.enabled => {
                let reqwest_client = HttpClientBuilder::get_reqwest_client_builder()?;
                RustyPipe::builder()
                    // Skip writing `rustypipe_reports/*.json` files to disk. YT Music
                    // playlists routinely include tracks with partial metadata (no
                    // album, no artist channel id), which rustypipe treats as
                    // WRN-level report-worthy events even though Soundome already
                    // handles the resulting `None` gracefully. See also the WARN
                    // filter in `shared::utils::logs::init_logger`.
                    .no_reporter()
                    .build_with_client(reqwest_client)
                    .expect("Failed to create RustyPipe client with proxy")
            }
            _ => RustyPipe::builder()
                .no_reporter()
                .build()
                .expect("Failed to create RustyPipe client"),
        };

        Ok(Self { client })
    }

    // =================
    // Utils
    // =================

    /// Converts a MusicTrack into a Track
    async fn get_complete_track_from_music_track(&self, track: TrackItem) -> Track {
        let mut artists: Vec<MusicArtist> = Vec::new();
        for artist in track.artists.iter() {
            // Some tracks (e.g. fetched from a playlist) carry an artist without a
            // channel id. Calling `music_artist` with an empty id always fails on
            // YT's side ("invalid data from YT: no header"), so skip the lookup
            // instead of letting rustypipe retry and write an error report for a
            // request that can never succeed.
            let Some(artist_id) = artist.id.as_deref().filter(|id| !id.is_empty()) else {
                continue;
            };
            if let Ok(artist) = self.client.query().music_artist(artist_id, false).await {
                artists.push(artist);
            }
        }

        // Same reasoning as above: an absent/empty album id always fails
        // `music_album`, so only issue the request when there is an id to look up.
        let album = match track.album.as_ref().map(|album| album.id.as_str()) {
            Some(album_id) if !album_id.is_empty() => {
                self.client.query().music_album(album_id).await.ok()
            }
            _ => None,
        };

        convert_track(track, artists, album)
    }

    /// Extracts the id from a youtube music track url (e.g: https://music.youtube.com/watch?v=U0ZoqmyGJo8&si=KsVobimXN6uao4s4 -> xxxxxxx)
    fn get_id_from_url(&self, url: &str) -> Option<String> {
        let re = Regex::new(Self::TRACK_REGEX).ok()?;
        let captures = re.captures(url).ok().flatten()?;
        captures.get(1).map(|m| m.as_str().to_string())
    }

    /// Extracts the id from a youtube music artist url (e.g: https://music.youtube.com/channel/UCfeJiV0Xu-C4z4DApRcznig -> xxxxxxx)
    fn get_artist_id_from_url(&self, url: &str) -> Option<String> {
        let re = Regex::new(Self::ARTIST_REGEX).ok()?;
        let captures = re.captures(url).ok().flatten()?;
        captures.get(1).map(|m| m.as_str().to_string())
    }

    /// Extracts the id from a youtube music album url (e.g: https://music.youtube.com/playlist?list=OLAK5uy_nEnkIMbtqesDReZnKM61c9Xo24Sgos8hA -> xxxxxxx)
    fn get_album_id_from_url(&self, url: &str) -> Option<String> {
        let re = Regex::new(Self::PLAYLIST_REGEX).ok()?;
        let captures = re.captures(url).ok().flatten()?;
        captures.get(1).map(|m| m.as_str().to_string())
    }

    /// Extracts the id from a youtube music playlist url (e.g: https://music.youtube.com/watch?v=YvI_FNrczzQ&list=RDCLAK5uy_mHkFNBTuR8DZUj61H5XY2onS7nRujVFx8 -> xxxxxxx)
    fn get_playlist_id_from_url(&self, url: &str) -> Option<String> {
        let re = Regex::new(Self::PLAYLIST_REGEX).ok()?;
        let captures = re.captures(url).ok().flatten()?;
        captures.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl Source for YoutubeMusic {
    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track> {
        let track_id = self
            .get_id_from_url(url)
            .ok_or(Error::InvalidUrl(url.to_string()))?;
        let track = self
            .client
            .query()
            .music_details(track_id)
            .await
            .map_err(|_| {
                Error::NotFound(format!("Youtube Music track from {}", url).to_string())
            })?;
        Ok(self.get_complete_track_from_music_track(track.track).await)
    }

    async fn get_tracks_from_query(&self, query: &str) -> SoundomeResult<Vec<Track>> {
        let results = self
            .client
            .query()
            .music_search_tracks(query)
            .await
            .map_err(mappers::convert_error)?;

        let tracks = self
            .enrich_tracks_with_limit(results.items.items.iter().cloned())
            .await;
        Ok(tracks)
    }

    async fn get_playlist_from_url(&self, url: &str) -> SoundomeResult<Playlist> {
        let playlist_id = self
            .get_playlist_id_from_url(url)
            .ok_or(Error::InvalidUrl(url.to_string()))?;
        let playlist = self
            .client
            .query()
            .music_playlist(playlist_id)
            .await
            .map_err(|_| Error::NotFound("Youtube Music playlist".to_string()))?;

        let cover = playlist.thumbnail.first().map(|t| t.url.clone());
        Ok(Playlist {
            id: None,
            name: playlist.name,
            source: Platform::YoutubeMusic,
            source_url: Some(url.to_string()),
            cover,
        })
    }

    async fn get_playlist_tracks_from_url(&self, _url: &str) -> SoundomeResult<Vec<PlaylistTrack>> {
        let playlist_id = self
            .get_playlist_id_from_url(_url)
            .ok_or(Error::InvalidUrl(_url.to_string()))?;
        let mut playlist = self
            .client
            .query()
            .music_playlist(playlist_id)
            .await
            .map_err(|_| Error::NotFound("Youtube Music playlist".to_string()))?;

        // Fetch all tracks using extend_limit to avoid 100-track limit
        playlist
            .tracks
            .extend_limit(self.client.query(), 10000)
            .await
            .map_err(|e| Error::Custom(format!("Failed to fetch all playlist tracks: {}", e)))?;

        tracing::info!(
            "Fetched {} tracks from YouTube Music playlist",
            playlist.tracks.items.len()
        );

        let tracks = self
            .enrich_tracks_with_limit(playlist.tracks.items.iter().cloned())
            .await;

        Ok(tracks
            .iter()
            .enumerate()
            .map(|(i, track)| PlaylistTrack {
                id: None,
                track: track.clone(),
                added_at: None,
                position: Some(i as u32),
            })
            .collect())
    }

    async fn get_artist_from_url(&self, url: &str) -> SoundomeResult<Artist> {
        let artist_id = self
            .get_artist_id_from_url(url)
            .ok_or(Error::InvalidUrl(url.to_string()))?;
        let artist = self
            .client
            .query()
            .music_artist(artist_id, true)
            .await
            .map_err(|_| {
                Error::NotFound(format!("Youtube Music artist from {}", url).to_string())
            })?;
        Ok(mappers::convert_artist(&artist))
    }

    async fn get_artist_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<Track>> {
        let artist_id = self
            .get_artist_id_from_url(url)
            .ok_or(Error::InvalidUrl(url.to_string()))?;

        // Fetch full discography: all albums for this artist
        let albums = self
            .client
            .query()
            .music_artist_albums(&artist_id, None, None)
            .await
            .map_err(|_| Error::NotFound(format!("Youtube Music artist albums from {}", url)))?;

        tracing::info!("Found {} albums for artist on YouTube Music", albums.len());

        // For each album, fetch all tracks
        let mut all_tracks: Vec<Track> = Vec::new();
        for album_item in &albums {
            let album = self.client.query().music_album(&album_item.id).await;

            match album {
                Ok(album) => {
                    let tracks = self
                        .enrich_tracks_with_limit(album.tracks.into_iter())
                        .await;
                    all_tracks.extend(tracks);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to fetch album {} ({}): {}",
                        album_item.name,
                        album_item.id,
                        e
                    );
                }
            }
        }

        tracing::info!(
            "Fetched {} tracks from artist discography on YouTube Music",
            all_tracks.len()
        );
        Ok(all_tracks)
    }

    async fn get_artists_from_query(&self, search: &str) -> SoundomeResult<Vec<Artist>> {
        let results = self
            .client
            .query()
            .music_search_artists(search)
            .await
            .map_err(mappers::convert_error)?;

        Ok(results
            .items
            .items
            .iter()
            .map(mappers::convert_artist_item)
            .collect())
    }

    async fn get_album_from_url(&self, url: &str) -> SoundomeResult<Album> {
        let album_id = self
            .get_album_id_from_url(url)
            .ok_or(Error::InvalidUrl(url.to_string()))?;
        let album = self
            .client
            .query()
            .music_album(album_id)
            .await
            .map_err(|_| {
                Error::NotFound(format!("Youtube Music album from {}", url).to_string())
            })?;
        Ok(mappers::convert_album(&album))
    }

    async fn get_albums_from_query(&self, search: &str) -> SoundomeResult<Vec<Album>> {
        let results = self
            .client
            .query()
            .music_search_albums(search)
            .await
            .map_err(mappers::convert_error)?;

        Ok(results
            .items
            .items
            .iter()
            .map(mappers::convert_album_item)
            .collect())
    }

    async fn get_album_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<Track>> {
        let album_id = self
            .get_album_id_from_url(url)
            .ok_or(Error::InvalidUrl(url.to_string()))?;
        let album = self
            .client
            .query()
            .music_album(album_id)
            .await
            .map_err(|_| Error::NotFound(format!("Youtube Music album from {}", url)))?;

        let tracks = self
            .enrich_tracks_with_limit(album.tracks.into_iter())
            .await;
        Ok(tracks)
    }

    async fn clean_track_metadata(&self, _track: &mut Track) -> SoundomeResult<()> {
        Ok(())
    }

    async fn clean_tracks_metadata(
        &self,
        _tracks: &mut Vec<&mut Track>,
        _on_batch: Option<&mut (dyn FnMut(usize, usize) + Send)>,
    ) -> SoundomeResult<()> {
        Ok(())
    }

    fn is_valid_track_url(url: &str) -> bool {
        let re = Regex::new(Self::TRACK_REGEX).unwrap(); // safe unwrap
        re.is_match(url).unwrap_or(false)
    }

    fn is_valid_playlist_url(url: &str) -> bool {
        let re = Regex::new(Self::PLAYLIST_REGEX).unwrap(); // safe unwrap
        re.is_match(url).unwrap_or(false)
    }

    fn is_valid_artist_url(url: &str) -> bool {
        let re = Regex::new(Self::ARTIST_REGEX).unwrap(); // safe unwrap
        re.is_match(url).unwrap_or(false)
    }

    fn is_valid_album_url(url: &str) -> bool {
        // YouTube Music album URLs contain "list=OLAK5uy_" (album playlist IDs)
        url.contains("music.youtube.com") && url.contains("list=OLAK5uy_")
    }
}
