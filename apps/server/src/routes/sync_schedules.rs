use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{delete, get, http::Status, patch, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::{database::Db, error::CustomError, response::Success};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncScheduleDto {
    pub id: i32,
    pub playlist_url: String,
    pub label: Option<String>,
    pub interval_seconds: i32,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub created_at: Option<String>,
}

impl SyncScheduleDto {
    fn from_model(s: shared::models::SyncSchedule) -> Option<Self> {
        Some(Self {
            id: s.id?,
            playlist_url: s.playlist_url,
            label: s.label,
            interval_seconds: s.interval_seconds,
            enabled: s.enabled,
            last_run: s.last_run.map(|t| t.to_string()),
            next_run: s.next_run.map(|t| t.to_string()),
            created_at: s.created_at.map(|t| t.to_string()),
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSyncScheduleBody {
    pub playlist_url: String,
    pub label: Option<String>,
    /// Interval in seconds. Defaults to 3600 (1 hour) if omitted.
    pub interval_seconds: Option<i32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSyncScheduleBody {
    pub label: Option<String>,
    pub interval_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

// ================================================================================================
// Routes
// ================================================================================================

/// List all sync schedules.
#[openapi]
#[get("/sync-schedules")]
pub async fn get_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<SyncScheduleDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let schedules = db
        .run(move |conn| services.sync_schedule_service.get_all(conn))
        .await
        .map_err(|e| crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: e.to_string(),
        }))?;
    Ok(Json(schedules.into_iter().filter_map(SyncScheduleDto::from_model).collect()))
}

/// Get a sync schedule by id.
#[openapi]
#[get("/sync-schedules/<id>")]
pub async fn get_by_id(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<SyncScheduleDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let schedule = db
        .run(move |conn| services.sync_schedule_service.get_by_id(conn, id))
        .await
        .map_err(|e| crate::utils::error::Error::Custom(CustomError {
            status: Status::NotFound,
            code: "NOT_FOUND".to_string(),
            message: e.to_string(),
        }))?;
    SyncScheduleDto::from_model(schedule).map(Json).ok_or_else(|| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: "Failed to map schedule".to_string(),
        })
    })
}

/// Create a new sync schedule.
#[openapi]
#[post("/sync-schedules", format = "json", data = "<body>")]
pub async fn create(
    body: Json<CreateSyncScheduleBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<SyncScheduleDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();
    let interval = body.interval_seconds.unwrap_or(3600);
    let schedule = db
        .run(move |conn| {
            services.sync_schedule_service.create(conn, body.playlist_url, body.label, interval)
        })
        .await
        .map_err(|e| crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: e.to_string(),
        }))?;
    SyncScheduleDto::from_model(schedule).map(Json).ok_or_else(|| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: "Failed to map schedule".to_string(),
        })
    })
}

/// Update a sync schedule (label, interval, or enabled flag).
#[openapi]
#[patch("/sync-schedules/<id>", format = "json", data = "<body>")]
pub async fn update(
    id: i32,
    body: Json<UpdateSyncScheduleBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<SyncScheduleDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    let schedule = db
        .run(move |conn| {
            let mut existing = services.sync_schedule_service.get_by_id(conn, id)?;
            if let Some(label) = body.label {
                existing.label = Some(label);
            }
            if let Some(interval) = body.interval_seconds {
                existing.interval_seconds = interval;
            }
            if let Some(enabled) = body.enabled {
                existing.enabled = enabled;
            }
            services.sync_schedule_service.update(conn, id, &existing)
        })
        .await
        .map_err(|e| crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: e.to_string(),
        }))?;
    SyncScheduleDto::from_model(schedule).map(Json).ok_or_else(|| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: "Failed to map schedule".to_string(),
        })
    })
}

/// Delete a sync schedule.
#[openapi]
#[delete("/sync-schedules/<id>")]
pub async fn delete(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.sync_schedule_service.delete(conn, id))
        .await
        .map_err(|e| crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: e.to_string(),
        }))?;
    Ok(Json(Success { success: true }))
}

/// Manually trigger a sync schedule immediately.
#[openapi]
#[post("/sync-schedules/<id>/trigger")]
pub async fn trigger(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<serde_json::Value>, crate::utils::error::Error> {
    let services_for_db = Arc::clone(services);
    let services_for_spawn = Arc::clone(services);

    // Fetch the schedule and mark it as ran immediately
    let schedule = db
        .run(move |conn| {
            let s = services_for_db.sync_schedule_service.get_by_id(conn, id)?;
            services_for_db.sync_schedule_service.mark_ran(conn, id)?;
            Ok::<_, shared::errors::Error>(s)
        })
        .await
        .map_err(|e| crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: e.to_string(),
        }))?;

    let url = schedule.playlist_url.clone();
    let label = schedule.label.clone();

    let task = db
        .run(move |conn| {
            services_for_spawn.task_service.create_playlist_sync(conn, &url, label)
        })
        .await
        .map_err(|e| crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: e.to_string(),
        }))?;

    let task_id = task.id.unwrap();
    let url = schedule.playlist_url.clone();
    crate::routes::download::spawn_playlist_sync_task(Arc::clone(services), task_id, url);

    Ok(Json(serde_json::json!({ "task_id": task_id })))
}
