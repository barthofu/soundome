use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{delete, get, http::Status, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::{
    DataQualityEntityType, DedupIgnoreEntry, DuplicateGroup, Platform, StructuralFinding,
    StructuralFindingEntity, StructuralFindingKind,
};

use crate::utils::{database::Db, error::CustomError, response::Success};

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

fn internal_error(err: impl std::fmt::Display) -> crate::utils::error::Error {
    crate::utils::error::Error::Custom(CustomError {
        status: Status::InternalServerError,
        code: "Internal".to_string(),
        message: err.to_string(),
    })
}
