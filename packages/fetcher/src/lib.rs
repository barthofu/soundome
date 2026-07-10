pub mod soundcloud;
pub mod spotify;
pub mod youtube;
pub mod youtube_music;

use async_trait::async_trait;
use config::Config;
use shared::errors::Error;
use shared::models::{Album, Artist, Platform, Playlist, PlaylistTrack, Track};
use shared::types::SoundomeResult;
use soundcloud::Soundcloud;
use spotify::Spotify;
use youtube::Youtube;
use youtube_music::YoutubeMusic;

/// Returns true when `key` is a known tracking/share-referral query parameter
/// (e.g. SoundCloud/Spotify/YouTube Music's `si` share id, or `utm_*` marketing
/// tags appended by browsers and social apps when a link is copied or shared).
///
/// This denylist is intentionally conservative: it only flags parameters we
/// know are never load-bearing, so it never risks dropping something a
/// platform actually needs (e.g. YouTube's `v`/`list`, which are never on
/// this list). Not exhaustive — extend it if a new tracking parameter shows
/// up in practice.
fn is_tracking_query_param(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.starts_with("utm_")
        || matches!(
            key.as_str(),
            "si" | "feature" | "fbclid" | "gclid" | "igshid"
        )
}

/// Curates a user-submitted source URL by stripping known tracking/share
/// query parameters (see [`is_tracking_query_param`]), leaving the rest of
/// the URL — including platform-essential parameters such as YouTube's
/// `v`/`list` — untouched.
///
/// This must run before a URL is used for `Source`/`Metadata` deduplication
/// or persisted as a `Playlist::source_url` / `Reference::external_url`:
/// two links that only differ by tracking noise (e.g. with vs without
/// `?si=...&utm_source=...`) need to curate to the exact same string,
/// otherwise `PlaylistService::upsert` and `TrackRepository::get_by_url`
/// treat them as different resources and create duplicates (see
/// `docs/workflows/download.md` — "URL-level deduplication").
///
/// ```
/// assert_eq!(
///     fetcher::curate_source_url(
///         "https://open.spotify.com/track/3vJQ0UFRHmNpZK8h7UmU1S?si=5ff30389104c4f19"
///     ),
///     "https://open.spotify.com/track/3vJQ0UFRHmNpZK8h7UmU1S"
/// );
/// ```
pub fn curate_source_url(url: &str) -> String {
    let Some((base, query)) = url.split_once('?') else {
        return url.to_string();
    };

    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .filter(|pair| {
            let key = pair.split('=').next().unwrap_or(pair);
            !is_tracking_query_param(key)
        })
        .collect();

    if kept.is_empty() {
        base.to_string()
    } else {
        format!("{base}?{}", kept.join("&"))
    }
}

#[async_trait]
pub trait Source {
    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track>;
    async fn get_tracks_from_query(&self, search: &str) -> SoundomeResult<Vec<Track>>;
    async fn get_playlist_from_url(&self, url: &str) -> SoundomeResult<Playlist>;
    async fn get_playlist_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<PlaylistTrack>>;
    async fn get_artist_from_url(&self, url: &str) -> SoundomeResult<Artist>;
    async fn get_artist_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<Track>>;
    async fn get_artists_from_query(&self, search: &str) -> SoundomeResult<Vec<Artist>>;
    async fn get_album_from_url(&self, url: &str) -> SoundomeResult<Album>;
    async fn get_albums_from_query(&self, search: &str) -> SoundomeResult<Vec<Album>>;
    async fn get_album_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<Track>>;

    /// Clean metadata of a single track
    async fn clean_track_metadata(&self, track: &mut Track) -> SoundomeResult<()>;
    /// Clean metadata (track title, artists names, etc) of multiple tracks.
    /// `on_batch` is invoked after each internal batch is processed with
    /// `(processed, total)`, so callers can surface live curation progress.
    async fn clean_tracks_metadata(
        &self,
        track: &mut Vec<&mut Track>,
        on_batch: Option<&mut (dyn FnMut(usize, usize) + Send)>,
    ) -> SoundomeResult<()>;

    fn is_valid_track_url(url: &str) -> bool;
    fn is_valid_playlist_url(url: &str) -> bool;
    fn is_valid_artist_url(url: &str) -> bool;
    fn is_valid_album_url(url: &str) -> bool;
}

pub struct Fetcher {
    spotify: Option<Spotify>,
    youtube: Option<Youtube>,
    youtube_music: Option<YoutubeMusic>,
    soundcloud: Option<Soundcloud>,
}

impl Fetcher {
    pub async fn new() -> Self {
        Self {
            spotify: Config::get().providers.spotify.as_ref().and_then(|cfg| {
                Spotify::new(&cfg.client_id, &cfg.client_secret)
                    .map_err(|e| {
                        tracing::error!("Failed to initialize Spotify source: {:?}", e);
                        e
                    })
                    .ok()
            }),
            youtube: Youtube::new()
                .map_err(|e| {
                    tracing::error!("Failed to initialize YouTube source: {:?}", e);
                    e
                })
                .ok(),
            youtube_music: YoutubeMusic::new()
                .map_err(|e| {
                    tracing::error!("Failed to initialize YouTube Music source: {:?}", e);
                    e
                })
                .ok(),
            soundcloud: Soundcloud::new()
                .await
                .map_err(|e| {
                    tracing::error!("Failed to initialize SoundCloud source: {:?}", e);
                    e
                })
                .ok(),
        }
    }
}

#[async_trait]
impl Source for Fetcher {
    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track> {
        match url {
            _ if Spotify::is_valid_track_url(url) => match &self.spotify {
                Some(spotify) => spotify.get_track_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
            },
            _ if Youtube::is_valid_track_url(url) => match &self.youtube {
                Some(youtube) => youtube.get_track_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Youtube.to_string())),
            },
            _ if YoutubeMusic::is_valid_track_url(url) => match &self.youtube_music {
                Some(youtube_music) => youtube_music.get_track_from_url(url).await,
                None => Err(Error::ProviderUnavailable(
                    Platform::YoutubeMusic.to_string(),
                )),
            },
            _ if Soundcloud::is_valid_track_url(url) => match &self.soundcloud {
                Some(soundcloud) => soundcloud.get_track_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
            },
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn get_playlist_from_url(&self, url: &str) -> SoundomeResult<Playlist> {
        match url {
            _ if Spotify::is_valid_playlist_url(url) => match &self.spotify {
                Some(spotify) => spotify.get_playlist_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
            },
            _ if Youtube::is_valid_playlist_url(url) => match &self.youtube {
                Some(youtube) => youtube.get_playlist_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Youtube.to_string())),
            },
            _ if YoutubeMusic::is_valid_playlist_url(url) => match &self.youtube_music {
                Some(youtube_music) => youtube_music.get_playlist_from_url(url).await,
                None => Err(Error::ProviderUnavailable(
                    Platform::YoutubeMusic.to_string(),
                )),
            },
            _ if Soundcloud::is_valid_playlist_url(url) => match &self.soundcloud {
                Some(soundcloud) => soundcloud.get_playlist_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
            },
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn get_playlist_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<PlaylistTrack>> {
        match url {
            _ if Spotify::is_valid_playlist_url(url) => match &self.spotify {
                Some(spotify) => spotify.get_playlist_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
            },
            _ if Youtube::is_valid_playlist_url(url) => match &self.youtube {
                Some(youtube) => youtube.get_playlist_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Youtube.to_string())),
            },
            _ if YoutubeMusic::is_valid_playlist_url(url) => match &self.youtube_music {
                Some(youtube_music) => youtube_music.get_playlist_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(
                    Platform::YoutubeMusic.to_string(),
                )),
            },
            _ if Soundcloud::is_valid_playlist_url(url) => match &self.soundcloud {
                Some(soundcloud) => soundcloud.get_playlist_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
            },
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn get_tracks_from_query(&self, _search: &str) -> SoundomeResult<Vec<Track>> {
        Err(Error::NotImplemented(
            "get_tracks_from_query is not implemented yet".to_string(),
        ))
    }

    async fn get_artist_from_url(&self, url: &str) -> SoundomeResult<Artist> {
        match url {
            _ if Spotify::is_valid_artist_url(url) => match &self.spotify {
                Some(spotify) => spotify.get_artist_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
            },
            _ if YoutubeMusic::is_valid_artist_url(url) => match &self.youtube_music {
                Some(youtube_music) => youtube_music.get_artist_from_url(url).await,
                None => Err(Error::ProviderUnavailable(
                    Platform::YoutubeMusic.to_string(),
                )),
            },
            _ if Soundcloud::is_valid_artist_url(url) => match &self.soundcloud {
                Some(soundcloud) => soundcloud.get_artist_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
            },
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn get_artist_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<Track>> {
        match url {
            _ if Spotify::is_valid_artist_url(url) => match &self.spotify {
                Some(spotify) => spotify.get_artist_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
            },
            _ if YoutubeMusic::is_valid_artist_url(url) => match &self.youtube_music {
                Some(youtube_music) => youtube_music.get_artist_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(
                    Platform::YoutubeMusic.to_string(),
                )),
            },
            _ if Soundcloud::is_valid_artist_url(url) => match &self.soundcloud {
                Some(soundcloud) => soundcloud.get_artist_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
            },
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn get_artists_from_query(&self, _search: &str) -> SoundomeResult<Vec<Artist>> {
        Err(Error::NotImplemented(
            "get_artists_from_query is not implemented yet".to_string(),
        ))
    }

    async fn get_album_from_url(&self, url: &str) -> SoundomeResult<Album> {
        match url {
            _ if Spotify::is_valid_album_url(url) => match &self.spotify {
                Some(spotify) => spotify.get_album_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
            },
            _ if YoutubeMusic::is_valid_album_url(url) => match &self.youtube_music {
                Some(youtube_music) => youtube_music.get_album_from_url(url).await,
                None => Err(Error::ProviderUnavailable(
                    Platform::YoutubeMusic.to_string(),
                )),
            },
            _ if Soundcloud::is_valid_album_url(url) => match &self.soundcloud {
                Some(soundcloud) => soundcloud.get_album_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
            },
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn get_albums_from_query(&self, _search: &str) -> SoundomeResult<Vec<Album>> {
        Err(Error::NotImplemented(
            "get_albums_from_query is not implemented yet".to_string(),
        ))
    }

    async fn get_album_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<Track>> {
        match url {
            _ if Spotify::is_valid_album_url(url) => match &self.spotify {
                Some(spotify) => spotify.get_album_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
            },
            _ if YoutubeMusic::is_valid_album_url(url) => match &self.youtube_music {
                Some(youtube_music) => youtube_music.get_album_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(
                    Platform::YoutubeMusic.to_string(),
                )),
            },
            _ if Soundcloud::is_valid_album_url(url) => match &self.soundcloud {
                Some(soundcloud) => soundcloud.get_album_tracks_from_url(url).await,
                None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
            },
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn clean_track_metadata(&self, track: &mut Track) -> SoundomeResult<()> {
        match track.get_source_platform() {
            Platform::SoundCloud => match &self.soundcloud {
                Some(soundcloud) => soundcloud.clean_track_metadata(track).await,
                None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
            },
            _ => Ok(()),
        }
    }

    async fn clean_tracks_metadata(
        &self,
        tracks: &mut Vec<&mut Track>,
        on_batch: Option<&mut (dyn FnMut(usize, usize) + Send)>,
    ) -> SoundomeResult<()> {
        match tracks.first() {
            Some(track) => match track.get_source_platform() {
                Platform::SoundCloud => match &self.soundcloud {
                    Some(soundcloud) => soundcloud.clean_tracks_metadata(tracks, on_batch).await,
                    None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
                },
                _ => Ok(()),
            },
            None => Ok(()),
        }
    }

    fn is_valid_track_url(url: &str) -> bool {
        Spotify::is_valid_track_url(url)
            || Youtube::is_valid_track_url(url)
            || YoutubeMusic::is_valid_track_url(url)
            || Soundcloud::is_valid_track_url(url)
    }

    fn is_valid_playlist_url(url: &str) -> bool {
        Spotify::is_valid_playlist_url(url)
            || Youtube::is_valid_playlist_url(url)
            || YoutubeMusic::is_valid_playlist_url(url)
            || Soundcloud::is_valid_playlist_url(url)
    }

    fn is_valid_artist_url(url: &str) -> bool {
        Spotify::is_valid_artist_url(url)
            || YoutubeMusic::is_valid_artist_url(url)
            || Soundcloud::is_valid_artist_url(url)
    }

    fn is_valid_album_url(url: &str) -> bool {
        Spotify::is_valid_album_url(url)
            || YoutubeMusic::is_valid_album_url(url)
            || Soundcloud::is_valid_album_url(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curates_soundcloud_playlist_url_with_tracking_params() {
        let url = "https://soundcloud.com/barthohm/sets/mentalcore?si=65669e64fec14d98b019534cd73d55c7&utm_source=clipboard&utm_medium=text&utm_campaign=social_sharing";
        assert_eq!(
            curate_source_url(url),
            "https://soundcloud.com/barthohm/sets/mentalcore"
        );
    }

    #[test]
    fn curates_spotify_track_url_with_share_id() {
        let url = "https://open.spotify.com/track/3vJQ0UFRHmNpZK8h7UmU1S?si=5ff30389104c4f19";
        assert_eq!(
            curate_source_url(url),
            "https://open.spotify.com/track/3vJQ0UFRHmNpZK8h7UmU1S"
        );
    }

    #[test]
    fn same_track_with_and_without_tracking_params_curates_identically() {
        let clean = "https://open.spotify.com/track/3vJQ0UFRHmNpZK8h7UmU1S";
        let dirty = "https://open.spotify.com/track/3vJQ0UFRHmNpZK8h7UmU1S?si=5ff30389104c4f19";
        assert_eq!(curate_source_url(clean), curate_source_url(dirty));
    }

    #[test]
    fn keeps_essential_youtube_query_param_and_drops_tracking() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&si=abc123";
        assert_eq!(
            curate_source_url(url),
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
    }

    #[test]
    fn keeps_essential_youtube_music_playlist_param_and_drops_tracking() {
        let url = "https://music.youtube.com/playlist?list=PLxyz123&si=abc123";
        assert_eq!(
            curate_source_url(url),
            "https://music.youtube.com/playlist?list=PLxyz123"
        );
    }

    #[test]
    fn leaves_url_without_query_string_untouched() {
        let url = "https://soundcloud.com/barthohm";
        assert_eq!(curate_source_url(url), url);
    }
}
