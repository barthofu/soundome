use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{delete, get, http::Status, patch, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::Artist;

use crate::routes::tracks::{reference_to_dto, AddReferenceBody, ReferenceDto};
use crate::utils::{database::Db, error::CustomError, response::Success};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct ArtistDto {
    pub id: i32,
    pub name: String,
    pub icon: Option<String>,
    pub references: Vec<ReferenceDto>,
}

impl ArtistDto {
    fn from_artist(artist: Artist) -> Option<Self> {
        Some(Self {
            id: artist.id?,
            name: artist.name,
            icon: artist.icon,
            references: artist
                .references
                .into_iter()
                .map(reference_to_dto)
                .collect(),
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateArtistBody {
    pub name: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MergeArtistsBody {
    /// Artist IDs to merge into `target_id`. These will be deleted after the merge.
    pub source_ids: Vec<i32>,
    /// The artist to keep.
    pub target_id: i32,
}

// ================================================================================================
// Routes
// ================================================================================================

#[openapi]
#[get("/artists")]
pub async fn get_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<ArtistDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.artist_service.get_all(conn))
        .await
        .map(|artists| {
            Json(
                artists
                    .into_iter()
                    .filter_map(ArtistDto::from_artist)
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
#[get("/artists/<id>")]
pub async fn get(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<ArtistDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.artist_service.get_by_id(conn, id))
        .await
        .and_then(|artist| {
            ArtistDto::from_artist(artist)
                .ok_or_else(|| shared::errors::Error::Database("Artist has no id".to_string()))
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

#[openapi]
#[patch("/artists/<id>", format = "application/json", data = "<body>")]
pub async fn update(
    id: i32,
    body: Json<UpdateArtistBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<ArtistDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    db.run(move |conn| {
        let mut artist = services.artist_service.get_by_id(conn, id)?;
        if let Some(name) = body.name {
            artist.name = name;
        }
        if let Some(icon) = body.icon {
            artist.icon = Some(icon);
        }
        services.artist_service.update(conn, id, &artist)
    })
    .await
    .and_then(|artist| {
        ArtistDto::from_artist(artist)
            .ok_or_else(|| shared::errors::Error::Database("Artist has no id".to_string()))
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
#[delete("/artists/<id>")]
pub async fn delete(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.artist_service.delete_by_id(conn, id))
        .await
        .map(|_| Json(Success { success: true }))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

#[openapi]
#[post("/artists/merge", format = "application/json", data = "<body>")]
pub async fn merge(
    body: Json<MergeArtistsBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<ArtistDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    if body.source_ids.is_empty() {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "source_ids must not be empty".to_string(),
        }));
    }
    if body.source_ids.contains(&body.target_id) {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "target_id must not appear in source_ids".to_string(),
        }));
    }

    db.run(move |conn| {
        services
            .artist_service
            .merge_into(conn, &body.source_ids, body.target_id)
    })
    .await
    .and_then(|artist| {
        ArtistDto::from_artist(artist)
            .ok_or_else(|| shared::errors::Error::Database("Artist has no id".to_string()))
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

// ================================================================================================
// Reference sub-resource
// ================================================================================================

/// List all references attached to an artist.
#[openapi]
#[get("/artists/<id>/references")]
pub async fn get_references(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<ReferenceDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.artist_service.get_by_id(conn, id))
        .await
        .map(|artist| {
            Json(
                artist
                    .references
                    .into_iter()
                    .map(reference_to_dto)
                    .collect(),
            )
        })
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::NotFound,
                code: "NotFound".to_string(),
                message: err.to_string(),
            })
        })
}

/// Add a reference to an artist.
#[openapi]
#[post(
    "/artists/<id>/references",
    format = "application/json",
    data = "<body>"
)]
pub async fn add_reference(
    id: i32,
    body: Json<AddReferenceBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<ReferenceDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let reference = body.into_inner().into_reference();

    db.run(move |conn| services.artist_service.add_reference(conn, id, reference))
        .await
        .map(|refs| Json(refs.into_iter().map(reference_to_dto).collect()))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

/// Remove a single reference from an artist by its reference row ID.
#[openapi]
#[delete("/artists/<_id>/references/<ref_id>")]
pub async fn delete_reference(
    _id: i32,
    ref_id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.artist_service.delete_reference(conn, ref_id))
        .await
        .map(|_| Json(Success { success: true }))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}
