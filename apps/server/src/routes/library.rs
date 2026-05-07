use std::{path::PathBuf, sync::Arc};

use config::Config;
use domain::services::{scan_service::ScanReport, ServiceLayer};
use rocket::{post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::utils::{database::Db, error::Error};

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
#[post("/api/library/scan", data = "<body>")]
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
