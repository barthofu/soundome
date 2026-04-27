use std::sync::Arc;

use domain::services::{track_service::ValidationPatch, ServiceLayer};
use rocket::{delete, get, http::Status, patch, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::{Reference, Track};

use crate::utils::{database::Db, error::CustomError};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct ArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AlbumDto {
    pub id: Option<i32>,
    pub title: String,
    pub artists: Vec<ArtistDto>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ReferenceDto {
    pub id: Option<i32>,
    pub ref_type: String,
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PendingValidationDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<ArtistDto>,
    pub album: Option<AlbumDto>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub file_path: Option<String>,
    pub validation_reason: Option<String>,
    pub references: Vec<ReferenceDto>,
}

impl PendingValidationDto {
    fn from_track(track: Track) -> Option<Self> {
        Some(Self {
            id: track.id?,
            title: track.title,
            artists: track
                .artists
                .into_iter()
                .map(|a| ArtistDto { id: a.id, name: a.name })
                .collect(),
            album: track.album.map(|a| AlbumDto {
                id: a.id,
                title: a.title,
                artists: a
                    .artists
                    .into_iter()
                    .map(|a| ArtistDto { id: a.id, name: a.name })
                    .collect(),
            }),
            date: track.date,
            genre: track.genre,
            cover: track.cover,
            duration: track.duration,
            track_number: track.track_number,
            disc_number: track.disc_number,
            label: track.label,
            file_path: track
                .file_path
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            validation_reason: track.validation_reason,
            references: track.references.into_iter().map(reference_to_dto).collect(),
        })
    }
}

fn reference_to_dto(r: Reference) -> ReferenceDto {
    ReferenceDto {
        id: r.id,
        ref_type: r.ref_type.as_ref().to_string(),
        platform: r.platform.as_ref().to_string(),
        external_id: r.external_id,
        external_url: r.external_url,
    }
}

// ================================================================================================
// Approve / Reject DTOs
// ================================================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApproveValidationBody {
    pub title: Option<String>,
    /// Replace track artists by name (comma-separated split handled client-side).
    pub artists: Option<Vec<String>>,
    pub album_title: Option<String>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
}

// ================================================================================================
// Routes
// ================================================================================================

#[openapi]
#[get("/validations")]
pub async fn get_pending(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<PendingValidationDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    db.run(move |conn| {
        services
            .track_service
            .get_pending_validations(conn)
    })
    .await
    .map(|tracks| {
        Json(
            tracks
                .into_iter()
                .filter_map(PendingValidationDto::from_track)
                .collect(),
        )
    })
    .map_err(|err| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: err.to_string(),
        })
    })
}

#[openapi]
#[patch("/validations/<id>", data = "<body>")]
pub async fn approve_validation(
    id: i32,
    body: Json<ApproveValidationBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<PendingValidationDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    db.run(move |conn| {
        let patch = ValidationPatch {
            title: body.title,
            artists: body.artists,
            album_title: body.album_title,
            genre: body.genre,
            date: body.date,
            track_number: body.track_number,
            disc_number: body.disc_number,
            label: body.label,
        };
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(services.download_service.finalize_validated_track(conn, id, patch))
        })
    })
    .await
    .and_then(|track| {
        PendingValidationDto::from_track(track)
            .ok_or_else(|| shared::errors::Error::Internal("track has no id after validation".into()))
    })
    .map(Json)
    .map_err(|err| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: err.to_string(),
        })
    })
}

#[openapi]
#[delete("/validations/<id>")]
pub async fn reject_validation(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<serde_json::Value>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    db.run(move |conn| services.track_service.delete_by_id(conn, id))
        .await
        .map(|_| Json(serde_json::json!({ "deleted": true })))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

#[openapi]
#[get("/tracks/recent?<limit>")]
pub async fn get_recent(
    limit: Option<i64>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<PendingValidationDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let limit = limit.unwrap_or(20).min(100);

    db.run(move |conn| {
        services
            .track_service
            .get_recent(conn, limit)
    })
    .await
    .map(|tracks| {
        Json(
            tracks
                .into_iter()
                .filter_map(PendingValidationDto::from_track)
                .collect(),
        )
    })
    .map_err(|err| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: err.to_string(),
        })
    })
}
