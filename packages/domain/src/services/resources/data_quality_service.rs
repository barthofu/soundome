use std::sync::Arc;

use diesel::SqliteConnection;
use shared::{
    models::{
        AiCleanupLogEntry, DataQualityEntityType, DedupIgnoreEntry, ReferenceAuditResult,
        StructuralFinding,
    },
    types::SoundomeResult,
};

use crate::ports::repositories::DataQualityRepository;

/// Orchestrates the "Data Quality" area: structural audits, the manual
/// duplicate-review ignore list, the remote reference audit cache, and the AI
/// cleanup change log. All logic that decides *what* counts as an anomaly
/// lives in the repository (it needs direct SQL access across the `_ref`
/// tables); this service is a thin pass-through so routes never talk to
/// `DataQualityRepository` directly.
pub struct DataQualityService {
    repo: Arc<dyn DataQualityRepository + Send + Sync>,
}

impl DataQualityService {
    pub fn new(repo: Arc<dyn DataQualityRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    /// Runs the structural audit (A-D): conflicting references, multiple
    /// platform references on one entity, platform/URL mismatches, and tracks
    /// missing a `Source` reference. Pure reads, safe to call on every page load.
    pub fn structural_findings(
        &self,
        conn: &mut SqliteConnection,
    ) -> SoundomeResult<Vec<StructuralFinding>> {
        self.repo.find_structural_findings(conn)
    }

    pub fn ignore_duplicate(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
        id_a: i32,
        id_b: i32,
    ) -> SoundomeResult<()> {
        self.repo.add_dedup_ignore(conn, entity_type, id_a, id_b)
    }

    pub fn list_ignored_duplicates(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
    ) -> SoundomeResult<Vec<DedupIgnoreEntry>> {
        self.repo.list_dedup_ignored(conn, entity_type)
    }

    pub fn record_reference_audit(
        &self,
        conn: &mut SqliteConnection,
        result: &ReferenceAuditResult,
    ) -> SoundomeResult<()> {
        self.repo.upsert_reference_audit(conn, result)
    }

    pub fn list_reference_audit(
        &self,
        conn: &mut SqliteConnection,
        entity_type: Option<DataQualityEntityType>,
    ) -> SoundomeResult<Vec<ReferenceAuditResult>> {
        self.repo.list_reference_audit(conn, entity_type)
    }

    pub fn log_ai_cleanup(
        &self,
        conn: &mut SqliteConnection,
        entry: &AiCleanupLogEntry,
    ) -> SoundomeResult<()> {
        self.repo.create_ai_cleanup_log(conn, entry)
    }

    pub fn list_ai_cleanup_log(
        &self,
        conn: &mut SqliteConnection,
        limit: i64,
    ) -> SoundomeResult<Vec<AiCleanupLogEntry>> {
        self.repo.list_ai_cleanup_log(conn, limit)
    }
}
