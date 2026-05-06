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
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "spotify" => Platform::Spotify,
            "soundcloud" => Platform::SoundCloud,
            "musicbrainz" => Platform::MusicBrainz,
            "youtube_music" => Platform::YoutubeMusic,
            "youtube" => Platform::Youtube,
            "bandcamp" => Platform::Bandcamp,
            _ => Platform::Unknown,
        }
    }

    pub fn from_string(s: String) -> Self {
        Platform::from_str(&s)
    }
}
