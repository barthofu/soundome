use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{get, http::Status, patch, post, serde::json::Json};
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

/// The single global cron configuration driving every scheduled sync
/// subscription in one pass (see `/sync-schedules`).
#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncSettingsDto {
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
}

impl SyncSettingsDto {
    fn from_model(s: shared::models::SyncSettings) -> Self {
        Self {
            cron_expression: s.cron_expression,
            enabled: s.enabled,
            last_run: s.last_run.map(|t| t.to_string()),
            next_run: s.next_run.map(|t| t.to_string()),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSyncSettingsBody {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
}

// ================================================================================================
// Routes
// ================================================================================================

/// Get the global scheduled sync cron configuration.
#[openapi]
#[get("/sync-settings")]
pub async fn get_settings(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<SyncSettingsDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.sync_settings_service.get(conn))
        .await
        .map(|s| Json(SyncSettingsDto::from_model(s)))
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: e.to_string(),
            })
        })
}

/// Update the global scheduled sync cron configuration (expression and/or
/// enabled flag).
#[openapi]
#[patch("/sync-settings", format = "json", data = "<body>")]
pub async fn update_settings(
    body: Json<UpdateSyncSettingsBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<SyncSettingsDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    // Validate the cron expression eagerly so a bad value never gets
    // persisted (the scheduler would otherwise fail silently every minute).
    if let Some(cron_expression) = &body.cron_expression {
        if let Err(e) =
            domain::schedule::calculate_next_run(chrono::Utc::now().naive_utc(), cron_expression)
        {
            return Err(crate::utils::error::Error::Custom(CustomError {
                status: Status::BadRequest,
                code: "BAD_REQUEST".to_string(),
                message: format!("Invalid cron expression: {e}"),
            }));
        }
    }

    db.run(move |conn| {
        let mut settings = services.sync_settings_service.get(conn)?;
        if let Some(cron_expression) = body.cron_expression {
            settings.cron_expression = cron_expression;
        }
        if let Some(enabled) = body.enabled {
            settings.enabled = enabled;
        }
        services.sync_settings_service.update(conn, &settings)
    })
    .await
    .map(|s| Json(SyncSettingsDto::from_model(s)))
    .map_err(|e| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "INTERNAL".to_string(),
            message: e.to_string(),
        })
    })
}

/// Immediately run a full scheduled-sync pass: enqueue every enabled
/// subscription now, without waiting for the global cron.
#[openapi]
#[post("/sync-settings/trigger")]
pub async fn trigger_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    registry: &rocket::State<Arc<CancellationRegistry>>,
    executor: &rocket::State<Arc<TaskExecutor>>,
) -> Result<Json<serde_json::Value>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let registry = Arc::clone(registry);
    let executor = Arc::clone(executor);

    let subscriptions = db
        .run({
            let services = Arc::clone(&services);
            move |conn| {
                services.sync_settings_service.mark_ran(conn)?;
                services.sync_schedule_service.get_enabled(conn)
            }
        })
        .await
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "INTERNAL".to_string(),
                message: e.to_string(),
            })
        })?;

    let mut task_ids = Vec::new();
    for subscription in subscriptions {
        let Some(subscription_id) = subscription.id else {
            continue;
        };
        let url = subscription.url.clone();
        let label = subscription.label.clone();
        let entity_type = subscription.entity_type;

        let services_for_task = Arc::clone(&services);
        let mark_ran_result = db
            .run(move |conn| {
                services_for_task
                    .sync_schedule_service
                    .mark_ran(conn, subscription_id)
            })
            .await;
        if let Err(e) = mark_ran_result {
            tracing::error!(
                "Trigger all: failed to mark subscription {} as ran: {}",
                subscription_id,
                e
            );
            continue;
        }

        let services_for_task = Arc::clone(&services);
        let task = db
            .run(move |conn| match entity_type {
                shared::models::SyncEntityType::Playlist => services_for_task
                    .task_service
                    .create_playlist_sync(conn, &url, label),
                shared::models::SyncEntityType::Artist => services_for_task
                    .task_service
                    .create_artist_sync(conn, &url, label),
            })
            .await;
        let task = match task {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(
                    "Trigger all: failed to create task for subscription {}: {}",
                    subscription_id,
                    e
                );
                continue;
            }
        };
        let Some(task_id) = task.id else { continue };
        let cancel_flag = registry.register(task_id);
        let url = subscription.url.clone();
        match subscription.entity_type {
            shared::models::SyncEntityType::Playlist => {
                executor.enqueue_playlist_sync(task_id, url, cancel_flag)
            }
            shared::models::SyncEntityType::Artist => {
                executor.enqueue_artist_sync(task_id, url, cancel_flag)
            }
        }
        task_ids.push(task_id);
    }

    Ok(Json(serde_json::json!({ "task_ids": task_ids })))
}
