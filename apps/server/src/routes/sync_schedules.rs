use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{delete, get, http::Status, patch, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::SyncEntityType;

use crate::utils::{
    cancellation::CancellationRegistry, database::Db, error::CustomError, response::Success,
    task_executor::TaskExecutor,
};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncScheduleDto {
    pub id: i32,
    pub entity_type: String,
    pub artist_id: Option<i32>,
    pub reference_id: Option<i32>,
    pub url: String,
    pub label: Option<String>,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub created_at: Option<String>,
}

impl SyncScheduleDto {
    fn from_model(s: shared::models::SyncSchedule) -> Option<Self> {
        Some(Self {
            id: s.id?,
            entity_type: s.entity_type.as_str().to_string(),
            artist_id: s.artist_id,
            reference_id: s.reference_id,
            url: s.url,
            label: s.label,
            enabled: s.enabled,
            last_run: s.last_run.map(|t| t.to_string()),
            created_at: s.created_at.map(|t| t.to_string()),
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSyncScheduleBody {
    /// Manual "add link" path: a raw URL, entity type auto-detected. Mutually
    /// exclusive with `artist_id`/`reference_id`.
    pub url: Option<String>,
    pub label: Option<String>,
    /// One-click "subscribe from the artist page" path: an artist + one of
    /// its `Source`/`Metadata` references (whichever carries a usable
    /// `external_url` for this artist — e.g. Spotify/SoundCloud artists use
    /// `Metadata`, YouTube Music artists use `Source`). Mutually exclusive
    /// with `url`.
    pub artist_id: Option<i32>,
    pub reference_id: Option<i32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSyncScheduleBody {
    pub label: Option<String>,
    pub enabled: Option<bool>,
}

// ================================================================================================
// Routes
// ================================================================================================

/// List all sync subscriptions.
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
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: e.to_string(),
            })
        })?;
    Ok(Json(
        schedules
            .into_iter()
            .filter_map(SyncScheduleDto::from_model)
            .collect(),
    ))
}

/// Get a sync subscription by id.
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
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::NotFound,
                code: "NOT_FOUND".to_string(),
                message: e.to_string(),
            })
        })?;
    SyncScheduleDto::from_model(schedule)
        .map(Json)
        .ok_or_else(|| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: "Failed to map schedule".to_string(),
            })
        })
}

/// Create a new sync subscription. Either supply `url` (manual "add link",
/// entity type auto-detected) or `artist_id` + `reference_id` (one-click
/// subscribe from the artist page, using one of the artist's `Source` or
/// `Metadata` references).
#[openapi]
#[post("/sync-schedules", format = "json", data = "<body>")]
pub async fn create(
    body: Json<CreateSyncScheduleBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<SyncScheduleDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let CreateSyncScheduleBody {
        url,
        label,
        artist_id,
        reference_id,
    } = body.into_inner();

    if url.is_some() && (artist_id.is_some() || reference_id.is_some()) {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BAD_REQUEST".to_string(),
            message: "Cannot provide both url and artist_id/reference_id".to_string(),
        }));
    }

    let schedule = if let (Some(artist_id), Some(reference_id)) = (artist_id, reference_id) {
        db.run(move |conn| {
            let artist = services.artist_service.get_by_id(conn, artist_id)?;
            let reference = artist
                .references
                .iter()
                .find(|r| {
                    r.id == Some(reference_id)
                        && domain::schedule::is_eligible_artist_sync_reference(r)
                })
                .cloned()
                .ok_or(shared::errors::Error::InvalidArg)?;
            let url = reference
                .external_url
                .clone()
                .ok_or(shared::errors::Error::InvalidArg)?;
            let label = label.or(Some(artist.name.clone()));
            services.sync_schedule_service.subscribe_artist_source(
                conn,
                artist_id,
                reference_id,
                url,
                label,
            )
        })
        .await
    } else if let Some(url) = url {
        db.run(move |conn| {
            let entity_type = domain::schedule::detect_sync_entity_type(&url);
            services
                .sync_schedule_service
                .subscribe_url(conn, entity_type, url, label)
        })
        .await
    } else {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BAD_REQUEST".to_string(),
            message: "Either url or (artist_id and reference_id) must be provided".to_string(),
        }));
    }
    .map_err(|e| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: e.to_string(),
        })
    })?;

    SyncScheduleDto::from_model(schedule)
        .map(Json)
        .ok_or_else(|| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: "Failed to map schedule".to_string(),
            })
        })
}

/// Update a sync subscription (label and/or enabled flag).
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
            if let Some(enabled) = body.enabled {
                existing.enabled = enabled;
            }
            services.sync_schedule_service.update(conn, id, &existing)
        })
        .await
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: e.to_string(),
            })
        })?;
    SyncScheduleDto::from_model(schedule)
        .map(Json)
        .ok_or_else(|| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: "Failed to map schedule".to_string(),
            })
        })
}

/// Delete a sync subscription (unsubscribe).
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
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: e.to_string(),
            })
        })?;
    Ok(Json(Success { success: true }))
}

/// Manually trigger a single sync subscription immediately (does not affect
/// the global cron's next_run).
#[openapi]
#[post("/sync-schedules/<id>/trigger")]
pub async fn trigger(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    registry: &rocket::State<Arc<CancellationRegistry>>,
    executor: &rocket::State<Arc<TaskExecutor>>,
) -> Result<Json<serde_json::Value>, crate::utils::error::Error> {
    let services_for_db = Arc::clone(services);
    let services_for_task = Arc::clone(services);
    let registry = Arc::clone(registry);
    let executor = Arc::clone(executor);

    // Fetch the subscription and mark it as ran immediately
    let schedule = db
        .run(move |conn| {
            let s = services_for_db.sync_schedule_service.get_by_id(conn, id)?;
            services_for_db.sync_schedule_service.mark_ran(conn, id)?;
            Ok::<_, shared::errors::Error>(s)
        })
        .await
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: e.to_string(),
            })
        })?;

    let url = schedule.url.clone();
    let label = schedule.label.clone();
    let entity_type = schedule.entity_type;

    let task = db
        .run(move |conn| match entity_type {
            SyncEntityType::Playlist => services_for_task
                .task_service
                .create_playlist_sync(conn, &url, label),
            SyncEntityType::Artist => services_for_task
                .task_service
                .create_artist_sync(conn, &url, label),
        })
        .await
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: e.to_string(),
            })
        })?;

    let task_id = task.id.unwrap();
    let url = schedule.url.clone();
    let cancel_flag = registry.register(task_id);
    match schedule.entity_type {
        SyncEntityType::Playlist => executor.enqueue_playlist_sync(task_id, url, cancel_flag),
        SyncEntityType::Artist => executor.enqueue_artist_sync(task_id, url, cancel_flag),
    }

    Ok(Json(serde_json::json!({ "task_id": task_id })))
}
