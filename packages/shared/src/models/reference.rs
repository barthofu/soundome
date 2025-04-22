use strum::{AsRefStr, Display};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, AsRefStr, PartialEq)]
pub enum ReferenceType {
    Source,
    Provider,
    Metadata,
    Reference,
}

impl ReferenceType {
    pub fn from_str(s: &str) -> Self {
        match s {
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

#[derive(Debug, Clone, Display, AsRefStr, PartialEq)]
pub enum Platform {
    Spotify,
    SoundCloud,
    MusicBrainz,
    YoutubeMusic,
    Youtube,
    Unknown
}

impl Platform {
    pub fn from_str(s: &str) -> Self {
        match s {
            "spotify" => Platform::Spotify,
            "soundcloud" => Platform::SoundCloud,
            "musicbrainz" => Platform::MusicBrainz,
            "youtube_music" => Platform::YoutubeMusic,
            "youtube" => Platform::Youtube,
            _ => Platform::Unknown,
        }
    }

    pub fn from_string(s: String) -> Self {
        Platform::from_str(&s)
    }
}
