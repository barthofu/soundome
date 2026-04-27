use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{get, http::Status, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;
use shared::models::Task;

use crate::utils::{database::Db, error::CustomError};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskDto {
    pub id: i32,
    pub task_type: String,
    pub status: String,
    pub label: Option<String>,
    pub progress: i32,
    pub total: Option<i32>,
    pub error: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl TaskDto {
    fn from_task(task: Task) -> Option<Self> {
        Some(Self {
            id: task.id?,
            task_type: task.task_type.as_ref().to_string(),
            status: task.status.as_ref().to_string(),
            label: task.label,
            progress: task.progress,
            total: task.total,
            error: task.error,
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
