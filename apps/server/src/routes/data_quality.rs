use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{get, http::Status, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;
use shared::models::{Platform, StructuralFinding, StructuralFindingEntity, StructuralFindingKind};

use crate::utils::{database::Db, error::CustomError};

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
