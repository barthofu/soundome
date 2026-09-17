use chrono::NaiveDateTime;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Platform;

/// The three library entity kinds the Data Quality area operates on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DataQualityEntityType {
    Artist,
    Album,
    Track,
}

impl DataQualityEntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataQualityEntityType::Artist => "artist",
            DataQualityEntityType::Album => "album",
            DataQualityEntityType::Track => "track",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "album" => DataQualityEntityType::Album,
            "track" => DataQualityEntityType::Track,
            _ => DataQualityEntityType::Artist,
        }
    }
}

// ================================================================================================
// Structural audit (no network calls — computed on demand from existing data)
// ================================================================================================

/// One of the entities involved in a `StructuralFinding`, for display purposes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StructuralFindingEntity {
    pub entity_type: DataQualityEntityType,
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StructuralFindingKind {
    /// The same `(platform, external_id/external_url)` reference is attached to
    /// two different entities. This is the exact fingerprint left behind by the
    /// SoundCloud artist-mapping bug fixed in `packages/fetcher/src/soundcloud`.
    ConflictingReference,
    /// The same entity has two references of the same `(platform, ref_type)`
    /// pointing at different `external_id`s — usually a sign of a botched merge
    /// or a reference that was attached to the wrong entity.
    MultiplePlatformReferences,
    /// A reference's `external_url` resolves (via `Platform::from_url`) to a
    /// different platform than the one stored on the reference itself.
    PlatformUrlMismatch,
    /// A track has no `Source` reference at all, which should never happen
    /// through the normal download pipeline.
    TrackMissingSourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StructuralFinding {
    pub kind: StructuralFindingKind,
    pub message: String,
    pub entities: Vec<StructuralFindingEntity>,
    pub platform: Option<Platform>,
    pub reference_id: Option<i32>,
}

// ================================================================================================
// Duplicate review — ignore list
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupIgnoreEntry {
    pub id: Option<i32>,
    pub entity_type: DataQualityEntityType,
    pub id_a: i32,
    pub id_b: i32,
    pub created_at: Option<NaiveDateTime>,
}

// ================================================================================================
// Remote reference audit (manual, network calls to providers)
// ================================================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceAuditStatus {
    /// Remote name matches the local name closely enough.
    Ok,
    /// Remote name diverges from the local name beyond the similarity threshold.
    Mismatch,
    /// The provider could not be reached or returned an error for this reference.
    Unreachable,
    /// This reference's platform/type is not supported by the remote audit yet.
    Unsupported,
}

impl ReferenceAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReferenceAuditStatus::Ok => "ok",
            ReferenceAuditStatus::Mismatch => "mismatch",
            ReferenceAuditStatus::Unreachable => "unreachable",
            ReferenceAuditStatus::Unsupported => "unsupported",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "mismatch" => ReferenceAuditStatus::Mismatch,
            "unreachable" => ReferenceAuditStatus::Unreachable,
            "unsupported" => ReferenceAuditStatus::Unsupported,
            _ => ReferenceAuditStatus::Ok,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceAuditResult {
    pub id: Option<i32>,
    pub entity_type: DataQualityEntityType,
    pub entity_id: i32,
    pub reference_id: i32,
    pub remote_name: Option<String>,
    pub similarity_score: Option<f64>,
    pub status: ReferenceAuditStatus,
    pub checked_at: Option<NaiveDateTime>,
}

// ================================================================================================
// AI cleanup log
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCleanupLogEntry {
    pub id: Option<i32>,
    pub track_id: Option<i32>,
    pub platform: Platform,
    pub source_external_id: Option<String>,
    pub before_title: String,
    pub before_artists: Vec<String>,
    pub after_title: String,
    pub after_artists: Vec<String>,
    pub rejected_artists: Vec<String>,
    pub created_at: Option<NaiveDateTime>,
}
