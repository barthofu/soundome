use std::sync::Arc;

use config::Config;
use domain::services::ServiceLayer;
use rocket::{http::Status, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

use crate::utils::{database::Db, error::CustomError};

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
        || url.contains("list=")
        || url.contains("/album/")
}

// ================================================================================================
// Routes
// ================================================================================================

/// Download a single track (synchronous) or start a background playlist sync (returns a task_id).
#[openapi]
#[post("/download", format = "json", data = "<body>")]
pub async fn download(
    body: Json<DownloadRequest>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<serde_json::Value>, crate::utils::error::Error> {
    let url = body.into_inner().url;
    let services = Arc::clone(services);

    if is_playlist_url(&url) {
        // --- Async path: create task, spawn background thread, return task_id immediately ---

        let url_clone = url.clone();
        let services_for_db = services.clone();
        let task = db
            .run(move |conn| {
                services_for_db.task_service.create_playlist_sync(conn, &url_clone, None)
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
        let db_url = Config::get().database.url.clone();

        // Spawn a dedicated OS thread with its own single-threaded Tokio runtime.
        // This avoids `Send` constraints on `&mut SqliteConnection` across `.await` points.
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build background tokio runtime");

            rt.block_on(async move {
                let conn = &mut database::init_connection(&db_url);

                if let Err(e) = services.task_service.set_running(conn, task_id) {
                    tracing::error!("Failed to set task {} as running: {}", task_id, e);
                }

                match services
                    .download_service
                    .sync_playlist_from_url(&url, conn, Some(task_id))
                    .await
                {
                    Ok(_) => {
                        if let Err(e) = services.task_service.set_completed(conn, task_id) {
                            tracing::error!("Failed to mark task {} as completed: {}", task_id, e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Playlist sync task {} failed: {}", task_id, e);
                        if let Err(e2) =
                            services.task_service.set_failed(conn, task_id, &e.to_string())
                        {
                            tracing::error!(
                                "Failed to mark task {} as failed: {}",
                                task_id,
                                e2
                            );
                        }
                    }
                }
            });
        });

        Ok(Json(serde_json::json!({
            "type": "playlist",
            "task_id": task_id,
        })))
    } else {
        // --- Sync path: single track download, block until done ---
        let track = db
            .run(move |conn| {
                tokio::task::block_in_place(|| {
                    Handle::current()
                        .block_on(services.download_service.download_track_from_url(&url, conn))
                })
            })
            .await
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
