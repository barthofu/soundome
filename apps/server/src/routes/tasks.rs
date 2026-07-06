use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{get, http::Status, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;
use shared::models::{Task, TaskStatus, TaskType};

use crate::utils::{cancellation::CancellationRegistry, database::Db, error::CustomError};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskTrackErrorDto {
    pub track: String,
    pub reason: String,
    pub provider_url: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskStatsDto {
    /// Tracks fully downloaded and moved to library.
    pub downloaded: i32,
    /// Tracks saved as "needs_validation" (staged, awaiting approval).
    pub to_validate: i32,
    /// Tracks already in library — linked but not re-downloaded.
    pub skipped: i32,
    /// Per-track failures that did not abort the whole sync.
    pub errors: Vec<TaskTrackErrorDto>,
    /// Live progress of an in-flight AI metadata curation phase, if one is running.
    pub ai_curation: Option<AiCurationProgressDto>,
}

/// Live progress of an AI metadata curation batch loop (title/artist cleanup).
#[derive(Debug, Serialize, JsonSchema)]
pub struct AiCurationProgressDto {
    /// Number of tracks processed so far across completed batches.
    pub processed: i32,
    /// Total number of tracks to curate in this phase.
    pub total: i32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskDto {
    pub id: i32,
    pub task_type: String,
    pub status: String,
    pub label: Option<String>,
    pub progress: i32,
    pub total: Option<i32>,
    pub error: Option<String>,
    pub stats: Option<TaskStatsDto>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl TaskDto {
    fn from_task(task: Task) -> Option<Self> {
        let stats = task.stats.map(|s| TaskStatsDto {
            downloaded: s.downloaded,
            to_validate: s.to_validate,
            skipped: s.skipped,
            errors: s
                .errors
                .into_iter()
                .map(|e| TaskTrackErrorDto {
                    track: e.track,
                    reason: e.reason,
                    provider_url: e.provider_url,
                })
                .collect(),
            ai_curation: s.ai_curation.map(|a| AiCurationProgressDto {
                processed: a.processed,
                total: a.total,
            }),
        });
        Some(Self {
            id: task.id?,
            task_type: task.task_type.as_ref().to_string(),
            status: task.status.as_ref().to_string(),
            label: task.label,
            progress: task.progress,
            total: task.total,
            error: task.error,
            stats,
            created_at: task.created_at.map(|t| t.to_string()),
            updated_at: task.updated_at.map(|t| t.to_string()),
        })
    }
}

// ================================================================================================
// Routes
// ================================================================================================

/// List all tasks (most recent first).
#[openapi]
#[get("/tasks")]
pub async fn get_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<TaskDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.task_service.get_all(conn))
        .await
        .map(|tasks| Json(tasks.into_iter().filter_map(TaskDto::from_task).collect()))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

/// Get a single task by ID (use for polling).
#[openapi]
#[get("/tasks/<id>")]
pub async fn get_by_id(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<TaskDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.task_service.get_by_id(conn, id))
        .await
        .and_then(|task| {
            TaskDto::from_task(task)
                .ok_or_else(|| shared::errors::Error::NotFound("Task has no id".to_string()))
        })
        .map(Json)
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::NotFound,
                code: "NotFound".to_string(),
                message: err.to_string(),
            })
        })
}

/// Retry a failed or interrupted task. Resets progress to 0 and re-spawns the background job.
/// Already-downloaded tracks will be skipped automatically.
#[openapi]
#[post("/tasks/<id>/retry")]
pub async fn retry(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    registry: &rocket::State<Arc<CancellationRegistry>>,
) -> Result<Json<TaskDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let services_for_spawn = services.clone();
    let registry = Arc::clone(registry);

    let task = db
        .run(move |conn| {
            let task = services.task_service.get_by_id(conn, id)?;

            // Allow retry for Failed, Running (stale), Pending (stuck), or Cancelled tasks
            if task.status != TaskStatus::Failed
                && task.status != TaskStatus::Running
                && task.status != TaskStatus::Pending
                && task.status != TaskStatus::Cancelled
            {
                return Err(shared::errors::Error::Custom(format!(
                    "Task {} is in status {:?} and cannot be retried",
                    id, task.status
                )));
            }

            services.task_service.reset_for_retry(conn, id)
        })
        .await
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::BadRequest,
                code: "RetryFailed".to_string(),
                message: err.to_string(),
            })
        })?;

    let task_id = task.id.expect("persisted task must have an id");

    // Extract URL from the task payload
    let url = extract_url_from_payload(&task.payload).ok_or_else(|| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "InvalidPayload".to_string(),
            message: format!("Task {} has no url in payload", task_id),
        })
    })?;

    let cancel_flag = registry.register(task_id);
    match task.task_type {
        TaskType::SyncPlaylist => {
            super::download::spawn_playlist_sync_task(
                services_for_spawn,
                task_id,
                url,
                cancel_flag,
                registry,
            );
        }
        TaskType::SyncArtist => {
            super::download::spawn_artist_sync_task(
                services_for_spawn,
                task_id,
                url,
                cancel_flag,
                registry,
            );
        }
        TaskType::SyncAlbum => {
            super::download::spawn_album_sync_task(
                services_for_spawn,
                task_id,
                url,
                cancel_flag,
                registry,
            );
        }
        _ => {
            return Err(crate::utils::error::Error::Custom(CustomError {
                status: Status::BadRequest,
                code: "UnsupportedTaskType".to_string(),
                message: format!("Retry is not supported for task type {:?}", task.task_type),
            }));
        }
    }

    TaskDto::from_task(task)
        .ok_or_else(|| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: "Task has no id".to_string(),
            })
        })
        .map(Json)
}

fn extract_url_from_payload(payload: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(payload)
        .ok()
        .and_then(|v| v.get("url")?.as_str().map(String::from))
}

/// Cancel a running task gracefully. The task will stop at the next track boundary.
#[openapi]
#[post("/tasks/<id>/cancel")]
pub async fn cancel(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    registry: &rocket::State<Arc<CancellationRegistry>>,
) -> Result<Json<TaskDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let registry = Arc::clone(registry);

    // Verify the task exists and is running
    let services_for_get = services.clone();
    let task = db
        .run(move |conn| services_for_get.task_service.get_by_id(conn, id))
        .await
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::NotFound,
                code: "NotFound".to_string(),
                message: err.to_string(),
            })
        })?;

    if task.status != TaskStatus::Running && task.status != TaskStatus::Pending {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "InvalidState".to_string(),
            message: format!(
                "Task {} is in status {:?} and cannot be cancelled",
                id, task.status
            ),
        }));
    }

    // Signal cancellation
    if !registry.cancel(id) {
        // Task not in registry (already finished between check and signal), update DB directly
        db.run(move |conn| services.task_service.set_cancelled(conn, id))
            .await
            .map_err(|err| {
                crate::utils::error::Error::Custom(CustomError {
                    status: Status::InternalServerError,
                    code: "Internal".to_string(),
                    message: err.to_string(),
                })
            })?;
    }

    // Return updated task state (may still show Running until the worker picks up the signal)
    let mut dto = TaskDto::from_task(task).ok_or_else(|| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: "Task has no id".to_string(),
        })
    })?;
    dto.status = "Cancelling".to_string();

    Ok(Json(dto))
}
