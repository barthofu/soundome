use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{http::Status, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::{
    cancellation::CancellationRegistry, database::Db, error::CustomError,
    task_executor::TaskExecutor,
};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DownloadRequest {
    pub url: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DownloadResult {
    pub title: String,
    pub artists: Vec<String>,
    pub needs_validation: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PlaylistDownloadResult {
    pub downloaded: usize,
    pub skipped: usize,
    pub failed: usize,
}

// ================================================================================================
// Helpers
// ================================================================================================

fn is_playlist_url(url: &str) -> bool {
    url.contains("/playlist/")
        || url.contains("/sets/")
        || (url.contains("list=") && !url.contains("list=OLAK5uy_"))
}

fn is_album_url(url: &str) -> bool {
    url.contains("open.spotify.com/album/")
        || (url.contains("music.youtube.com") && url.contains("list=OLAK5uy_"))
}

fn is_artist_url(url: &str) -> bool {
    url.contains("open.spotify.com/artist/")
        || url.contains("music.youtube.com/channel/")
        || (url.contains("soundcloud.com/")
            && !is_playlist_url(url)
            && !url.contains("/sets/")
            && url.trim_end_matches('/').split('/').count() == 4) // https://soundcloud.com/username
}

// ================================================================================================
// Routes
// ================================================================================================

/// Download a single track (synchronous), or start a background playlist/artist sync (returns a task_id).
///
/// **Concurrency**: every download is dispatched through the shared serial
/// executor (see `utils/task_executor.rs`). At most one job runs at a time,
/// which prevents SQLite lock contention and keeps external API usage below
/// rate-limit thresholds. Batch jobs (playlist/artist/album) return a
/// `task_id` immediately and run in the background; single-track downloads
/// block the HTTP response until the queue reaches them.
#[openapi]
#[post("/download", format = "json", data = "<body>")]
pub async fn download(
    body: Json<DownloadRequest>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    registry: &rocket::State<Arc<CancellationRegistry>>,
    executor: &rocket::State<Arc<TaskExecutor>>,
) -> Result<Json<serde_json::Value>, crate::utils::error::Error> {
    let url = body.into_inner().url;
    let services = Arc::clone(services);
    let registry = Arc::clone(registry);
    let executor = Arc::clone(executor);

    if is_playlist_url(&url) {
        // --- Async path: create task, enqueue, return task_id immediately ---

        let url_clone = url.clone();
        let services_for_db = services.clone();
        let task = db
            .run(move |conn| {
                services_for_db
                    .task_service
                    .create_playlist_sync(conn, &url_clone, None)
            })
            .await
            .map_err(|err| {
                crate::utils::error::Error::Custom(CustomError {
                    status: Status::InternalServerError,
                    code: "TaskCreateFailed".to_string(),
                    message: err.to_string(),
                })
            })?;

        let task_id = task.id.expect("created task must have an id");
        let cancel_flag = registry.register(task_id);
        executor.enqueue_playlist_sync(task_id, url, cancel_flag);

        Ok(Json(serde_json::json!({
            "type": "playlist",
            "task_id": task_id,
        })))
    } else if is_artist_url(&url) {
        // --- Async path: artist sync, create task, enqueue ---

        let url_clone = url.clone();
        let services_for_db = services.clone();
        let task = db
            .run(move |conn| {
                services_for_db
                    .task_service
                    .create_artist_sync(conn, &url_clone, None)
            })
            .await
            .map_err(|err| {
                crate::utils::error::Error::Custom(CustomError {
                    status: Status::InternalServerError,
                    code: "TaskCreateFailed".to_string(),
                    message: err.to_string(),
                })
            })?;

        let task_id = task.id.expect("created task must have an id");
        let cancel_flag = registry.register(task_id);
        executor.enqueue_artist_sync(task_id, url, cancel_flag);

        Ok(Json(serde_json::json!({
            "type": "artist",
            "task_id": task_id,
        })))
    } else if is_album_url(&url) {
        // --- Async path: album sync, create task, enqueue ---

        let url_clone = url.clone();
        let services_for_db = services.clone();
        let task = db
            .run(move |conn| {
                services_for_db
                    .task_service
                    .create_album_sync(conn, &url_clone, None)
            })
            .await
            .map_err(|err| {
                crate::utils::error::Error::Custom(CustomError {
                    status: Status::InternalServerError,
                    code: "TaskCreateFailed".to_string(),
                    message: err.to_string(),
                })
            })?;

        let task_id = task.id.expect("created task must have an id");
        let cancel_flag = registry.register(task_id);
        executor.enqueue_album_sync(task_id, url, cancel_flag);

        Ok(Json(serde_json::json!({
            "type": "album",
            "task_id": task_id,
        })))
    } else {
        // --- Sync path: single track download, enqueue and block until done ---
        // Routing through the executor guarantees the queue invariant even for
        // synchronous downloads (never bypasses the serial worker).
        let rx = executor.enqueue_single_track(url);
        let track = rx
            .await
            .map_err(|_| {
                crate::utils::error::Error::Custom(CustomError {
                    status: Status::InternalServerError,
                    code: "TaskExecutorClosed".to_string(),
                    message: "Task executor dropped the request before completion".to_string(),
                })
            })?
            .map_err(|err| {
                crate::utils::error::Error::Custom(CustomError {
                    status: Status::InternalServerError,
                    code: "DownloadFailed".to_string(),
                    message: err.to_string(),
                })
            })?;

        Ok(Json(serde_json::json!({
            "type": "track",
            "title": track.title,
            "artists": track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
            "needs_validation": track.needs_validation,
        })))
    }
}
