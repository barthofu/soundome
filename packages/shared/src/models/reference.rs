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
}
