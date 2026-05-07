use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use diesel::SqliteConnection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::{models::Track, types::SoundomeResult};
use walkdir::WalkDir;

use crate::ports::repositories::TrackRepository;

// ================================================================================================
// Result types
// ================================================================================================

/// Category of a scan result for a single file or DB row.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanCategory {
    /// File found at expected path; tags consistent.
    Ok,
    /// File found via `SOUNDOME_ID` but at a different path — `file_path` in DB will be updated.
    PathChanged,
    /// File found but tags diverge from DB — flagged `needs_validation`.
    TagConflict,
    /// DB row exists but file not found anywhere.
    Missing,
    /// File has `SOUNDOME_ID` but no matching DB row.
    Orphan,
    /// File matched via MusicBrainz ID; no `SOUNDOME_ID` present.
    LegacyMatch,
    /// File exists in library but is unknown to Soundome.
    Unmanaged,
}

/// A single entry in the `ScanReport`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScanEntry {
    pub category: ScanCategory,
    /// Absolute path of the audio file on disk (if present).
    pub file_path: Option<String>,
    /// DB track id (if matched).
    pub track_id: Option<i32>,
    /// Human-readable title from the DB row or file tag.
    pub title: Option<String>,
}

/// Aggregated result returned after a scan run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScanReport {
    pub dry_run: bool,
    pub library_root: String,
    pub entries: Vec<ScanEntry>,

    // Convenience counts
    pub ok: usize,
    pub path_changed: usize,
    pub tag_conflict: usize,
    pub missing: usize,
    pub orphan: usize,
    pub legacy_match: usize,
    pub unmanaged: usize,
    /// Number of `path_changed` entries that were automatically updated in the DB.
    pub paths_updated: usize,
    /// Number of `tag_conflict` entries that were flagged `needs_validation` in the DB.
    pub conflicts_flagged: usize,
}

impl ScanReport {
    fn new(dry_run: bool, library_root: String) -> Self {
        Self {
            dry_run,
            library_root,
            entries: Vec::new(),
            ok: 0,
            path_changed: 0,
            tag_conflict: 0,
            missing: 0,
            orphan: 0,
            legacy_match: 0,
            unmanaged: 0,
            paths_updated: 0,
            conflicts_flagged: 0,
        }
    }

    fn push(&mut self, entry: ScanEntry) {
        match entry.category {
            ScanCategory::Ok => self.ok += 1,
            ScanCategory::PathChanged => self.path_changed += 1,
            ScanCategory::TagConflict => self.tag_conflict += 1,
            ScanCategory::Missing => self.missing += 1,
            ScanCategory::Orphan => self.orphan += 1,
            ScanCategory::LegacyMatch => self.legacy_match += 1,
            ScanCategory::Unmanaged => self.unmanaged += 1,
        }
        self.entries.push(entry);
    }
}

// ================================================================================================
// Service
// ================================================================================================

pub struct ScanService {
    track_repo: Arc<dyn TrackRepository + Send + Sync>,
}

impl ScanService {
    pub fn new(track_repo: Arc<dyn TrackRepository + Send + Sync>) -> Self {
        Self { track_repo }
    }

    /// Walk `library_root` and reconcile the filesystem against the database.
    ///
    /// When `dry_run` is `false`:
    /// - `path_changed` entries have their `file_path` updated in the DB.
    /// - `tag_conflict` entries are flagged `needs_validation = true`.
    ///
    /// When `dry_run` is `true`, no mutations are made.
    pub fn scan(
        &self,
        conn: &mut SqliteConnection,
        library_root: &Path,
        dry_run: bool,
    ) -> SoundomeResult<ScanReport> {
        let library_root_str = library_root.to_string_lossy().to_string();
        let mut report = ScanReport::new(dry_run, library_root_str);

        // Load all finalized tracks into an in-memory index keyed by SOUNDOME_ID.
        let finalized = self.track_repo.get_all_finalized(conn)?;
        let mut by_soundome_id: HashMap<String, Track> = HashMap::new();
        let mut by_path: HashMap<String, Track> = HashMap::new();
        // Tracks the SOUNDOME_IDs of files that were classified in Phase 1.
        let mut visited_soundome_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        // Tracks the absolute paths of files that were classified in Phase 1.
        let mut visited_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

        for track in &finalized {
            if let Some(ref sid) = track.soundome_id {
                by_soundome_id.insert(sid.clone(), track.clone());
            }
            if let Some(ref fp) = track.file_path {
                by_path.insert(fp.to_string_lossy().to_string(), track.clone());
            }
        }

        // ── Phase 1: walk the library and classify each audio file ──────────────────

        let audio_extensions = ["mp3", "flac", "m4a", "mp4", "aac", "ogg", "opus", "wav"];

        for entry in WalkDir::new(library_root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            if !audio_extensions.contains(&ext.as_str()) {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();

            // Read SOUNDOME_ID from file
            let file_soundome_id = tagger::file::read_soundome_id_from_file(&PathBuf::from(path));

            if let Some(ref sid) = file_soundome_id {
                visited_soundome_ids.insert(sid.clone());
                visited_paths.insert(path_str.clone());

                if let Some(db_track) = by_soundome_id.get(sid) {
                    let db_path = db_track
                        .file_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if db_path != path_str {
                        // Path changed — file was moved/renamed
                        let entry = ScanEntry {
                            category: ScanCategory::PathChanged,
                            file_path: Some(path_str.clone()),
                            track_id: db_track.id,
                            title: Some(db_track.title.clone()),
                        };
                        report.push(entry);

                        if !dry_run {
                            if let Some(id) = db_track.id {
                                let mut updated = db_track.clone();
                                updated.file_path = Some(PathBuf::from(&path_str));
                                match self.track_repo.update(conn, id, &updated) {
                                    Ok(_) => {
                                        report.paths_updated += 1;
                                        tracing::info!(
                                            "Updated file_path for track {} → {}",
                                            id,
                                            path_str
                                        );
                                    }
                                    Err(e) => tracing::warn!(
                                        "Failed to update file_path for track {}: {}",
                                        id,
                                        e
                                    ),
                                }
                            }
                        }
                    } else {
                        // Check for tag divergence (title comparison is a lightweight proxy)
                        let file_track_result =
                            tagger::file::get_track_from_file(&PathBuf::from(path));

                        let has_conflict = match &file_track_result {
                            Ok(ft) => tags_diverge(db_track, ft),
                            Err(_) => false,
                        };

                        if has_conflict {
                            let entry = ScanEntry {
                                category: ScanCategory::TagConflict,
                                file_path: Some(path_str.clone()),
                                track_id: db_track.id,
                                title: Some(db_track.title.clone()),
                            };
                            report.push(entry);

                            if !dry_run {
                                if let Some(id) = db_track.id {
                                    let mut updated = db_track.clone();
                                    updated.needs_validation = true;
                                    updated.validation_reason =
                                        Some("file_tag_divergence".to_string());
                                    match self.track_repo.update(conn, id, &updated) {
                                        Ok(_) => {
                                            report.conflicts_flagged += 1;
                                            tracing::info!(
                                                "Flagged track {} for validation (tag conflict)",
                                                id
                                            );
                                        }
                                        Err(e) => tracing::warn!(
                                            "Failed to flag track {} for validation: {}",
                                            id,
                                            e
                                        ),
                                    }
                                }
                            }
                        } else {
                            report.push(ScanEntry {
                                category: ScanCategory::Ok,
                                file_path: Some(path_str),
                                track_id: db_track.id,
                                title: Some(db_track.title.clone()),
                            });
                        }
                    }
                } else {
                    // Has SOUNDOME_ID but no DB row — orphan
                    report.push(ScanEntry {
                        category: ScanCategory::Orphan,
                        file_path: Some(path_str),
                        track_id: None,
                        title: None,
                    });
                }
            } else {
                visited_paths.insert(path_str.clone());
                // No SOUNDOME_ID — try to match by stored path
                if let Some(db_track) = by_path.get(&path_str) {
                    // File is known by path but lacks SOUNDOME_ID (legacy or external tag edit)
                    report.push(ScanEntry {
                        category: ScanCategory::LegacyMatch,
                        file_path: Some(path_str),
                        track_id: db_track.id,
                        title: Some(db_track.title.clone()),
                    });
                } else {
                    // Completely unknown file
                    report.push(ScanEntry {
                        category: ScanCategory::Unmanaged,
                        file_path: Some(path_str),
                        track_id: None,
                        title: None,
                    });
                }
            }
        }

        // ── Phase 2: find DB rows whose files are missing ────────────────────────────

        for track in &finalized {
            let fp = match &track.file_path {
                Some(p) => p.clone(),
                None => continue,
            };

            let fp_str = fp.to_string_lossy().to_string();

            // Skip tracks whose file was already found (by SOUNDOME_ID or path) in Phase 1.
            let found_by_soundome_id = track
                .soundome_id
                .as_deref()
                .is_some_and(|sid| visited_soundome_ids.contains(sid));
            let found_by_path = visited_paths.contains(&fp_str);

            if found_by_soundome_id || found_by_path {
                continue;
            }

            // File not found anywhere during the walk.
            report.push(ScanEntry {
                category: ScanCategory::Missing,
                file_path: Some(fp_str),
                track_id: track.id,
                title: Some(track.title.clone()),
            });
        }

        tracing::info!(
            "Scan complete: ok={} path_changed={} tag_conflict={} missing={} orphan={} legacy={} unmanaged={}",
            report.ok,
            report.path_changed,
            report.tag_conflict,
            report.missing,
            report.orphan,
            report.legacy_match,
            report.unmanaged,
        );

        Ok(report)
    }
}

// ================================================================================================
// Helpers
// ================================================================================================

/// Returns `true` when the tags in `file_track` diverge from the DB `db_track` on
/// key metadata fields (title and primary artist name).
///
/// **Note**: This is a lightweight heuristic; it only compares the two most
/// user-visible fields to avoid false positives from harmless formatting
/// differences (e.g. track-number padding).  More fields can be added as needed
/// once the false-positive rate is understood in practice.
fn tags_diverge(db_track: &Track, file_track: &Track) -> bool {
    let title_differs = normalize(&db_track.title) != normalize(&file_track.title);

    let db_artist = db_track
        .artists
        .first()
        .map(|a| normalize(&a.name))
        .unwrap_or_default();
    let file_artist = file_track
        .artists
        .first()
        .map(|a| normalize(&a.name))
        .unwrap_or_default();
    let artist_differs =
        !db_artist.is_empty() && !file_artist.is_empty() && db_artist != file_artist;

    title_differs || artist_differs
}

fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}
