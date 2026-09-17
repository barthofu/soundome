use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{ai_cleanup_log, dedup_ignore, reference_audit_cache};

// ================================================================================================
// Dedup ignore list
// ================================================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Serialize)]
#[diesel(table_name = dedup_ignore)]
pub struct DedupIgnoreEntity {
    pub id: i32,
    pub entity_type: String,
    pub id_a: i32,
    pub id_b: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = dedup_ignore)]
pub struct NewDedupIgnoreEntity {
    pub entity_type: String,
    pub id_a: i32,
    pub id_b: i32,
}

// ================================================================================================
// Reference audit cache
// ================================================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Serialize)]
#[diesel(table_name = reference_audit_cache)]
pub struct ReferenceAuditCacheEntity {
    pub id: i32,
    pub entity_type: String,
    pub entity_id: i32,
    pub reference_id: i32,
    pub remote_name: Option<String>,
    pub similarity_score: Option<f64>,
    pub status: String,
    pub checked_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = reference_audit_cache)]
pub struct NewReferenceAuditCacheEntity {
    pub entity_type: String,
    pub entity_id: i32,
    pub reference_id: i32,
    pub remote_name: Option<String>,
    pub similarity_score: Option<f64>,
    pub status: String,
}

// ================================================================================================
// AI cleanup log
// ================================================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Serialize)]
#[diesel(table_name = ai_cleanup_log)]
pub struct AiCleanupLogEntity {
    pub id: i32,
    pub track_id: Option<i32>,
    pub platform: String,
    pub source_external_id: Option<String>,
    pub before_title: String,
    pub before_artists: String,
    pub after_title: String,
    pub after_artists: String,
    pub rejected_artists: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = ai_cleanup_log)]
pub struct NewAiCleanupLogEntity {
    pub track_id: Option<i32>,
    pub platform: String,
    pub source_external_id: Option<String>,
    pub before_title: String,
    pub before_artists: String,
    pub after_title: String,
    pub after_artists: String,
    pub rejected_artists: Option<String>,
}
