use std::sync::Arc;

use diesel::Connection as _;
use domain::services::ServiceLayer;
use rocket::{delete, get, http::Status, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::{
    DataQualityEntityType, DedupIgnoreEntry, DuplicateGroup, Platform, ReferenceAuditView,
    StructuralFinding, StructuralFindingEntity, StructuralFindingKind, TaskStatus, TaskType,
};

use crate::utils::{
    cancellation::CancellationRegistry, database::Db, error::CustomError, response::Success,
    task_executor::TaskExecutor,
};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct StructuralFindingEntityDto {
    pub entity_type: String,
    pub id: i32,
    pub name: String,
}

impl From<StructuralFindingEntity> for StructuralFindingEntityDto {
    fn from(e: StructuralFindingEntity) -> Self {
        Self {
            entity_type: e.entity_type.as_str().to_string(),
            id: e.id,
            name: e.name,
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StructuralFindingDto {
    pub kind: String,
    pub message: String,
    pub entities: Vec<StructuralFindingEntityDto>,
    pub platform: Option<String>,
    pub reference_id: Option<i32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IgnoreDuplicateBody {
    pub id_a: i32,
    pub id_b: i32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DedupIgnoreDto {
    pub entity_type: String,
    pub id_a: i32,
    pub id_b: i32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RemoteAuditTaskDto {
    pub task_id: i32,
}

impl From<DedupIgnoreEntry> for DedupIgnoreDto {
    fn from(entry: DedupIgnoreEntry) -> Self {
        Self {
            entity_type: entry.entity_type.as_str().to_string(),
            id_a: entry.id_a,
            id_b: entry.id_b,
        }
    }
}

impl From<StructuralFinding> for StructuralFindingDto {
    fn from(f: StructuralFinding) -> Self {
        Self {
            kind: kind_as_str(f.kind).to_string(),
            message: f.message,
            entities: f.entities.into_iter().map(Into::into).collect(),
            platform: f.platform.map(|p: Platform| p.as_ref().to_string()),
            reference_id: f.reference_id,
        }
    }
}

fn kind_as_str(kind: StructuralFindingKind) -> &'static str {
    match kind {
        StructuralFindingKind::ConflictingReference => "conflicting_reference",
        StructuralFindingKind::MultiplePlatformReferences => "multiple_platform_references",
        StructuralFindingKind::PlatformUrlMismatch => "platform_url_mismatch",
        StructuralFindingKind::TrackMissingSourceReference => "track_missing_source_reference",
    }
}

fn parse_entity_type(entity_type: &str) -> Option<DataQualityEntityType> {
    match entity_type {
        "artists" | "artist" => Some(DataQualityEntityType::Artist),
        "albums" | "album" => Some(DataQualityEntityType::Album),
        "tracks" | "track" => Some(DataQualityEntityType::Track),
        _ => None,
    }
}

// ================================================================================================
// Routes
// ================================================================================================

/// Runs the structural reference audit (A-D): conflicting references shared
/// across entities, multiple platform references on the same entity,
/// platform/URL mismatches, and tracks missing a `Source` reference. Pure
/// reads over existing data, no network calls — safe to call on every page
/// load of the Data Quality > Reference Audit > Structural tab.
#[openapi]
#[get("/data-quality/audit/structural")]
pub async fn get_structural_findings(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<StructuralFindingDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.data_quality_service.structural_findings(conn))
        .await
        .map(|findings| Json(findings.into_iter().map(Into::into).collect()))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

/// Lists connected components of similar library entities, excluding pairs
/// the user has previously marked as "not a duplicate".
#[openapi]
#[get("/data-quality/duplicates/<entity_type>")]
pub async fn get_duplicate_groups(
    entity_type: &str,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<DuplicateGroup>>, crate::utils::error::Error> {
    let Some(entity_type) = parse_entity_type(entity_type) else {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "entity_type must be artists, albums, or tracks".to_string(),
        }));
    };
    let services = Arc::clone(services);
    db.run(move |conn| {
        services
            .data_quality_service
            .duplicate_groups(conn, entity_type)
    })
    .await
    .map(Json)
    .map_err(internal_error)
}

/// Lists persisted "not a duplicate" pairs for a review queue tab.
#[openapi]
#[get("/data-quality/duplicates/<entity_type>/ignored")]
pub async fn get_ignored_duplicates(
    entity_type: &str,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<DedupIgnoreDto>>, crate::utils::error::Error> {
    let Some(entity_type) = parse_entity_type(entity_type) else {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "entity_type must be artists, albums, or tracks".to_string(),
        }));
    };
    let services = Arc::clone(services);
    db.run(move |conn| {
        services
            .data_quality_service
            .list_ignored_duplicates(conn, entity_type)
    })
    .await
    .map(|entries| Json(entries.into_iter().map(Into::into).collect()))
    .map_err(internal_error)
}

/// Persist one pair that the user has confirmed is not a duplicate.
#[openapi]
#[post(
    "/data-quality/duplicates/<entity_type>/ignore",
    format = "json",
    data = "<body>"
)]
pub async fn ignore_duplicate(
    entity_type: &str,
    body: Json<IgnoreDuplicateBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let Some(entity_type) = parse_entity_type(entity_type) else {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "entity_type must be artists, albums, or tracks".to_string(),
        }));
    };
    let body = body.into_inner();
    if body.id_a == body.id_b {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "id_a and id_b must be different".to_string(),
        }));
    }
    let services = Arc::clone(services);
    db.run(move |conn| {
        services
            .data_quality_service
            .ignore_duplicate(conn, entity_type, body.id_a, body.id_b)
    })
    .await
    .map(|_| Json(Success { success: true }))
    .map_err(internal_error)
}

/// Undo a previous "not a duplicate" decision.
#[openapi]
#[delete("/data-quality/duplicates/<entity_type>/ignore/<id_a>/<id_b>")]
pub async fn restore_ignored_duplicate(
    entity_type: &str,
    id_a: i32,
    id_b: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let Some(entity_type) = parse_entity_type(entity_type) else {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "entity_type must be artists, albums, or tracks".to_string(),
        }));
    };
    let services = Arc::clone(services);
    db.run(move |conn| {
        services
            .data_quality_service
            .restore_ignored_duplicate(conn, entity_type, id_a, id_b)
    })
    .await
    .map(|_| Json(Success { success: true }))
    .map_err(internal_error)
}

/// Starts the remote provider audit as one serialized background task. This
/// endpoint is manual-only: no scheduler or page-load path enqueues it.
#[openapi]
#[post("/data-quality/audit/remote/run")]
pub async fn start_remote_reference_audit(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
    registry: &rocket::State<Arc<CancellationRegistry>>,
    executor: &rocket::State<Arc<TaskExecutor>>,
) -> Result<Json<RemoteAuditTaskDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let task = db
        .run(move |conn| {
            conn.transaction(|tx| {
                let active = services.task_service.get_all(tx)?.into_iter().any(|task| {
                    task.task_type == TaskType::ReferenceAudit
                        && (task.status == TaskStatus::Pending
                            || task.status == TaskStatus::Running)
                });
                if active {
                    return Ok(None);
                }
                services.task_service.create_reference_audit(tx).map(Some)
            })
        })
        .await
        .map_err(internal_error)?;

    let Some(task) = task else {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::Conflict,
            code: "AuditAlreadyRunning".to_string(),
            message: "A remote reference audit is already pending or running".to_string(),
        }));
    };
    let task_id = task.id.ok_or_else(|| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: "Created remote audit task has no id".to_string(),
        })
    })?;
    let cancel_flag = registry.register(task_id);
    executor.enqueue_reference_audit(task_id, cancel_flag);
    Ok(Json(RemoteAuditTaskDto { task_id }))
}

/// Returns cached remote audit outcomes enriched with the current local entity
/// name and the audited reference URL/type.
#[openapi]
#[get("/data-quality/audit/remote")]
pub async fn get_remote_reference_audit(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<ReferenceAuditView>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| {
        services
            .data_quality_service
            .list_reference_audit_views(conn)
    })
    .await
    .map(Json)
    .map_err(internal_error)
}

/// Apply the provider's current remote name/title to the local entity.
#[openapi]
#[post("/data-quality/audit/remote/<audit_id>/apply")]
pub async fn apply_remote_reference_name(
    audit_id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| {
        services
            .data_quality_service
            .apply_remote_reference_name(conn, audit_id)
    })
    .await
    .map(|_| Json(Success { success: true }))
    .map_err(internal_error)
}

/// Dismiss the audit result while preserving it in the history/cache.
#[openapi]
#[post("/data-quality/audit/remote/<audit_id>/dismiss")]
pub async fn dismiss_remote_reference_audit(
    audit_id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| {
        services
            .data_quality_service
            .dismiss_remote_reference_audit(conn, audit_id)
    })
    .await
    .map(|_| Json(Success { success: true }))
    .map_err(internal_error)
}

/// Delete the audited entity reference from the library and remove its cached result.
#[openapi]
#[post("/data-quality/audit/remote/<audit_id>/delete-reference")]
pub async fn delete_audited_reference(
    audit_id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| {
        services
            .data_quality_service
            .delete_audited_reference(conn, audit_id)
    })
    .await
    .map(|_| Json(Success { success: true }))
    .map_err(internal_error)
}

fn internal_error(err: impl std::fmt::Display) -> crate::utils::error::Error {
    crate::utils::error::Error::Custom(CustomError {
        status: Status::InternalServerError,
        code: "Internal".to_string(),
        message: err.to_string(),
    })
}
