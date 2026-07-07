use std::{path::PathBuf, sync::Arc};

use config::Config;
use domain::services::{scan_service::ScanReport, ServiceLayer};
use rocket::{get, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use walkdir::WalkDir;

use crate::utils::{
    cancellation::CancellationRegistry, database::Db, error::Error,
    task_executor::TaskExecutor,
};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScanRequest {
    /// Override the library root to scan. Defaults to `general.base_library_dir`.
    pub library_root: Option<String>,
    /// When `true`, no mutations are applied to the database.
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IngestRequest {
    /// Path to the audio file to ingest.
    ///
    /// - If the path is **absolute**, it is used as-is.
    /// - If the path is **relative** (or just a filename), it is resolved against
    ///   `general.ingest_dir` from the configuration.
    pub file_path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct IngestResult {
    pub title: String,
    pub artists: Vec<String>,
    pub needs_validation: bool,
}

/// A single audio file entry reported by `GET /api/library/ingest/files`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct IngestFileEntry {
    /// Filename only (no directory component).
    pub name: String,
    /// Absolute path on the server filesystem.
    pub path: String,
    /// Path relative to `ingest_dir` (includes sub-directory components).
    pub relative_path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Audio tags read from the file (best-effort; `null` if unreadable).
    pub tags: Option<IngestFileTags>,
}

/// Subset of audio tags extracted from a file in the ingest directory.
#[derive(Debug, Serialize, JsonSchema)]
pub struct IngestFileTags {
    pub title: Option<String>,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub duration_secs: Option<u32>,
    pub track_number: Option<u32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct IngestFilesResponse {
    /// Absolute path of the directory that was listed.
    pub ingest_dir: String,
    pub files: Vec<IngestFileEntry>,
}

// ================================================================================================
// Routes
// ================================================================================================

/// Scan the library directory and reconcile the filesystem against the database.
///
/// Returns a `ScanReport` that categorises every audio file found into one of:
/// `ok`, `path_changed`, `tag_conflict`, `missing`, `orphan`, `legacy_match`, `unmanaged`.
///
/// When `dry_run` is `false` (the default):
/// - `path_changed` entries automatically update `file_path` in the database.
/// - `tag_conflict` entries are flagged `needs_validation = true`.
#[openapi(tag = "library")]
#[post("/library/scan", data = "<body>")]
pub async fn scan(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    body: Json<ScanRequest>,
) -> Result<Json<ScanReport>, Error> {
    let services = Arc::clone(services);
    let library_root = body
        .library_root
        .clone()
        .unwrap_or_else(|| Config::get().general.base_library_dir.clone());
    let dry_run = body.dry_run;

    let report = db
        .run(move |conn| {
            services
                .scan_service
                .scan(conn, &PathBuf::from(&library_root), dry_run)
        })
        .await
        .map_err(Error::from)?;

    Ok(Json(report))
}

/// Ingest a single local audio file into the library.
///
/// Reads the embedded tags, enriches via MusicBrainz, deduplicates against existing
/// tracks, and — when confident — tags, moves, and persists the file.
///
/// When metadata quality is insufficient, the track is persisted as
/// `needs_validation = true` and the file is staged for manual review.
///
/// **Path resolution**: if `file_path` is relative (or a bare filename), it is
/// resolved against `general.ingest_dir`; absolute paths are used as-is.
#[openapi(tag = "library")]
#[post("/library/ingest", data = "<body>")]
pub async fn ingest(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    body: Json<IngestRequest>,
) -> Result<Json<IngestResult>, Error> {
    let services = Arc::clone(services);

    // Resolve the file path: relative paths are anchored to ingest_dir.
    let raw = PathBuf::from(&body.file_path);
    let file_path = if raw.is_absolute() {
        raw
    } else {
        PathBuf::from(&Config::get().general.ingest_dir).join(raw)
    };

    let track = db
        .run(move |conn| {
            tokio::task::block_in_place(|| {
                Handle::current().block_on(
                    services
                        .download_service
                        .ingest_local_file(conn, &file_path),
                )
            })
        })
        .await
        .map_err(Error::from)?;

    Ok(Json(IngestResult {
        title: track.title.clone(),
        artists: track.artists.iter().map(|a| a.name.clone()).collect(),
        needs_validation: track.needs_validation,
    }))
}

/// List all audio files currently present in the configured `ingest_dir`,
/// including files in sub-directories (recursive walk).
///
/// Returns file names, absolute paths, relative paths, sizes, and best-effort
/// audio tags read from each file. No database interaction.
#[openapi(tag = "library")]
#[get("/library/ingest/files")]
pub async fn list_ingest_files() -> Result<Json<IngestFilesResponse>, Error> {
    let ingest_dir = Config::get().general.ingest_dir.clone();
    let dir = PathBuf::from(&ingest_dir);

    // Resolve to an absolute path for display and for building canonical file paths.
    let ingest_dir_display = dir
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(ingest_dir);

    let audio_extensions = ["mp3", "flac", "m4a", "mp4", "aac", "ogg", "opus", "wav"];

    let files = tokio::task::spawn_blocking(move || {
        let mut entries: Vec<IngestFileEntry> = Vec::new();
        if !dir.exists() {
            return entries;
        }

        // Canonicalize the root so all returned paths are true absolute paths,
        // regardless of whether `ingest_dir` was configured as a relative path.
        let canon_dir = dir.canonicalize().unwrap_or(dir.clone());

        for walk_entry in WalkDir::new(&canon_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = walk_entry.path().to_path_buf();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            if !audio_extensions.contains(&ext.as_str()) {
                continue;
            }

            let size_bytes = walk_entry.metadata().map(|m| m.len()).unwrap_or(0);

            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let relative_path = path
                .strip_prefix(&canon_dir)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| name.clone());

            // Best-effort tag read — never fail the whole listing for one bad file.
            let tags = tagger::file::get_track_from_file(&path)
                .ok()
                .map(|t| IngestFileTags {
                    title: if t.title.is_empty() {
                        None
                    } else {
                        Some(t.title)
                    },
                    artists: t.artists.into_iter().map(|a| a.name).collect(),
                    album: t.album.map(|a| a.title),
                    date: t.date,
                    genre: t.genre,
                    duration_secs: t.duration.map(|d| d as u32),
                    track_number: t.track_number.map(|n| n as u32),
                });

            entries.push(IngestFileEntry {
                name,
                path: path.to_string_lossy().to_string(),
                relative_path,
                size_bytes,
                tags,
            });
        }

        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        entries
    })
    .await
    .unwrap_or_default();

    Ok(Json(IngestFilesResponse {
        ingest_dir: ingest_dir_display,
        files,
    }))
}

/// Start a background task that ingests all audio files in `ingest_dir`.
///
/// Returns a `task_id` immediately. Poll `GET /api/tasks/:id` for progress and stats.
///
/// **Concurrency**: dispatched through the shared serial executor — will wait
/// in queue behind any playlist/artist/album sync currently in progress.
#[openapi(tag = "library")]
#[post("/library/ingest/all")]
pub async fn ingest_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    registry: &rocket::State<Arc<CancellationRegistry>>,
    executor: &rocket::State<Arc<TaskExecutor>>,
) -> Result<Json<serde_json::Value>, Error> {
    let services = Arc::clone(services);
    let registry = Arc::clone(registry);
    let executor = Arc::clone(executor);
    let ingest_dir = Config::get().general.ingest_dir.clone();

    let services_for_db = services.clone();
    let ingest_dir_clone = ingest_dir.clone();
    let task = db
        .run(move |conn| {
            services_for_db
                .task_service
                .create_ingest_dir(conn, &ingest_dir_clone)
        })
        .await
        .map_err(Error::from)?;

    let task_id = task.id.expect("created task must have an id");
    // Ingest currently has no cancellation checkpoint, but we register a flag
    // anyway to keep the registry consistent (and future-proof for cancellation).
    let _cancel_flag = registry.register(task_id);

    executor.enqueue_ingest_dir(task_id, PathBuf::from(ingest_dir));

    Ok(Json(serde_json::json!({ "task_id": task_id })))
}
