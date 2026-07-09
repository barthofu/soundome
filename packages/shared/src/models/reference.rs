use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Reference {
    pub id: Option<i32>,
    pub ref_type: ReferenceType,
    pub platform: Platform,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

impl Reference {
    /// Best-effort `(platform, external_id)` inference from a bare URL, so callers
    /// (mainly the manual "add reference" UI) only need to supply a link and a
    /// `ReferenceType` instead of picking the platform and copying an ID by hand.
    ///
    /// This is pure string parsing — no network access or provider credentials
    /// required — mirroring the lightweight URL-to-id conventions already used by
    /// `packages/fetcher` (e.g. Spotify's "last path segment" rule). SoundCloud and
    /// Bandcamp URLs do not embed a stable id (SoundCloud permalinks and Bandcamp
    /// subdomains both require an authenticated API call to resolve), so only the
    /// platform is inferred for those and `external_id` stays `None`. The reference
    /// remains fully usable via `external_url` alone in that case.
    pub fn infer_platform_and_id(url: &str) -> (Platform, Option<String>) {
        let platform = Platform::from_url(url);
        let external_id = match platform {
            Platform::Spotify | Platform::MusicBrainz => Self::last_path_segment(url),
            Platform::Youtube => {
                Self::query_param(url, "v").or_else(|| Self::last_path_segment(url))
            }
            Platform::YoutubeMusic => Self::query_param(url, "v").or_else(|| {
                Self::strip_query(url)
                    .split("/channel/")
                    .nth(1)
                    .map(|s| s.to_string())
            }),
            Platform::SoundCloud | Platform::Bandcamp | Platform::Unknown => None,
        };
        (platform, external_id)
    }

    /// Strips query string and fragment from a URL.
    fn strip_query(url: &str) -> &str {
        url.split(['?', '#']).next().unwrap_or(url)
    }

    /// Returns the last non-empty path segment, ignoring any query string/fragment.
    /// Works for URLs like `.../track/{id}`, `.../artist/{id}?si=...`, etc.
    fn last_path_segment(url: &str) -> Option<String> {
        let cleaned = Self::strip_query(url).trim_end_matches('/');
        let segment = cleaned.rsplit('/').next()?;
        if segment.is_empty() {
            None
        } else {
            Some(segment.to_string())
        }
    }

    /// Extracts a single query parameter's value (e.g. `v` from `?v=abc&list=xyz`).
    fn query_param(url: &str, key: &str) -> Option<String> {
        let query = url.split('?').nth(1)?.split('#').next()?;
        query.split('&').find_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next()?;
            let v = parts.next()?;
            (k == key && !v.is_empty()).then(|| v.to_string())
        })
    }
}

// ================================================================================================
// Enums
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, PartialEq, JsonSchema)]
pub enum ReferenceType {
    Source,
    Provider,
    Metadata,
    Reference,
}

impl ReferenceType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "source" => ReferenceType::Source,
            "provider" => ReferenceType::Provider,
            "metadata" => ReferenceType::Metadata,
            "reference" => ReferenceType::Reference,
            _ => ReferenceType::Source,
        }
    }

    pub fn from_string(s: String) -> Self {
        ReferenceType::from_str(&s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, AsRefStr, PartialEq, JsonSchema)]
pub enum Platform {
    Spotify,
    SoundCloud,
    MusicBrainz,
    YoutubeMusic,
    Youtube,
    Bandcamp,
    Unknown,
}

impl Platform {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        // Normalize away underscores: DB/repository code round-trips this value
        // through `Platform::X.as_ref().to_lowercase()` (e.g. `YoutubeMusic` ->
        // "youtubemusic", no separator), while some call sites still pass the
        // historical snake_case spelling ("youtube_music"). Stripping `_` lets
        // both forms (and the raw PascalCase variant name) resolve correctly
        // instead of silently falling back to `Unknown`.
        match s.to_lowercase().replace('_', "").as_str() {
            "spotify" => Platform::Spotify,
            "soundcloud" => Platform::SoundCloud,
            "musicbrainz" => Platform::MusicBrainz,
            "youtubemusic" => Platform::YoutubeMusic,
            "youtube" => Platform::Youtube,
            "bandcamp" => Platform::Bandcamp,
            _ => Platform::Unknown,
        }
    }

    pub fn from_string(s: String) -> Self {
        Platform::from_str(&s)
    }

    /// Best-effort platform detection from a URL's host. Used to auto-fill the
    /// platform when a user adds a reference by pasting a link. Order matters:
    /// `music.youtube.com` must be checked before the plain `youtube.com` suffix
    /// match, since it also ends with that substring.
    pub fn from_url(url: &str) -> Self {
        let host = url
            .split("://")
            .nth(1)
            .unwrap_or(url)
            .split(['/', '?', '#'])
            .next()
            .unwrap_or("")
            .to_lowercase();

        if host.ends_with("music.youtube.com") {
            Platform::YoutubeMusic
        } else if host.ends_with("spotify.com") {
            Platform::Spotify
        } else if host.ends_with("soundcloud.com") {
            Platform::SoundCloud
        } else if host.ends_with("musicbrainz.org") {
            Platform::MusicBrainz
        } else if host.ends_with("youtube.com") || host.ends_with("youtu.be") {
            Platform::Youtube
        } else if host.ends_with("bandcamp.com") {
            Platform::Bandcamp
        } else {
            Platform::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_spotify_track_id() {
        let (platform, id) = Reference::infer_platform_and_id(
            "https://open.spotify.com/track/3n3Ppam7vgaVa1iaRUc9Lp?si=abc",
        );
        assert_eq!(platform, Platform::Spotify);
        assert_eq!(id.as_deref(), Some("3n3Ppam7vgaVa1iaRUc9Lp"));
    }

    #[test]
    fn infers_spotify_artist_id_with_locale_prefix() {
        let (platform, id) = Reference::infer_platform_and_id(
            "https://open.spotify.com/intl-fr/artist/06HL4z0CvFAxyc27GXpf02",
        );
        assert_eq!(platform, Platform::Spotify);
        assert_eq!(id.as_deref(), Some("06HL4z0CvFAxyc27GXpf02"));
    }

    #[test]
    fn infers_youtube_video_id_from_watch_url() {
        let (platform, id) = Reference::infer_platform_and_id(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PL123",
        );
        assert_eq!(platform, Platform::Youtube);
        assert_eq!(id.as_deref(), Some("dQw4w9WgXcQ"));
    }

    #[test]
    fn infers_youtube_video_id_from_short_url() {
        let (platform, id) =
            Reference::infer_platform_and_id("https://youtu.be/dQw4w9WgXcQ?si=abc");
        assert_eq!(platform, Platform::Youtube);
        assert_eq!(id.as_deref(), Some("dQw4w9WgXcQ"));
    }

    #[test]
    fn infers_youtube_music_track_id() {
        let (platform, id) = Reference::infer_platform_and_id(
            "https://music.youtube.com/watch?v=U0ZoqmyGJo8&si=xyz",
        );
        assert_eq!(platform, Platform::YoutubeMusic);
        assert_eq!(id.as_deref(), Some("U0ZoqmyGJo8"));
    }

    #[test]
    fn infers_youtube_music_channel_id() {
        let (platform, id) = Reference::infer_platform_and_id(
            "https://music.youtube.com/channel/UCfeJiV0Xu-C4z4DApRcznig",
        );
        assert_eq!(platform, Platform::YoutubeMusic);
        assert_eq!(id.as_deref(), Some("UCfeJiV0Xu-C4z4DApRcznig"));
    }

    #[test]
    fn infers_musicbrainz_id() {
        let (platform, id) = Reference::infer_platform_and_id(
            "https://musicbrainz.org/artist/5b11f4ce-a62d-471e-81fc-a69a8278c7da",
        );
        assert_eq!(platform, Platform::MusicBrainz);
        assert_eq!(id.as_deref(), Some("5b11f4ce-a62d-471e-81fc-a69a8278c7da"));
    }

    #[test]
    fn infers_soundcloud_platform_without_id() {
        let (platform, id) =
            Reference::infer_platform_and_id("https://soundcloud.com/artist/track-name");
        assert_eq!(platform, Platform::SoundCloud);
        assert_eq!(id, None);
    }

    #[test]
    fn infers_bandcamp_platform_without_id() {
        let (platform, id) =
            Reference::infer_platform_and_id("https://artist.bandcamp.com/track/song");
        assert_eq!(platform, Platform::Bandcamp);
        assert_eq!(id, None);
    }

    #[test]
    fn unknown_url_yields_unknown_platform_and_no_id() {
        let (platform, id) = Reference::infer_platform_and_id("https://example.com/whatever");
        assert_eq!(platform, Platform::Unknown);
        assert_eq!(id, None);
    }
}
