use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{delete, get, http::Status, patch, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::Album;

use crate::utils::{database::Db, error::CustomError, response::Success};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct AlbumArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AlbumDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<AlbumArtistDto>,
    pub album_type: String,
    pub cover: Option<String>,
    pub date: Option<String>,
}

impl AlbumDto {
    fn from_album(album: Album) -> Option<Self> {
        Some(Self {
            id: album.id?,
            title: album.title,
            artists: album
                .artists
                .into_iter()
                .map(|a| AlbumArtistDto { id: a.id, name: a.name })
                .collect(),
            album_type: album.album_type.as_ref().to_string(),
            cover: album.cover,
            date: album.date,
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAlbumBody {
    pub title: Option<String>,
    pub date: Option<String>,
}

// ================================================================================================
// Routes
// ================================================================================================

#[openapi]
#[get("/albums")]
pub async fn get_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<AlbumDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.album_service.get_all(conn))
        .await
        .map(|albums| Json(albums.into_iter().filter_map(AlbumDto::from_album).collect()))
        .map_err(|err| crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: err.to_string(),
        }))
}

#[openapi]
#[get("/albums/<id>")]
pub async fn get(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<AlbumDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.album_service.get_by_id(conn, id))
        .await
        .and_then(|album| {
            AlbumDto::from_album(album)
                .ok_or_else(|| shared::errors::Error::Database("Album has no id".to_string()))
        })
        .map(Json)
        .map_err(|err| crate::utils::error::Error::Custom(CustomError {
            status: Status::NotFound,
            code: "NotFound".to_string(),
            message: err.to_string(),
        }))
}

#[openapi]
#[patch("/albums/<id>", format = "application/json", data = "<body>")]
pub async fn update(
    id: i32,
    body: Json<UpdateAlbumBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<AlbumDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    db.run(move |conn| {
        let mut album = services.album_service.get_by_id(conn, id)?;
        if let Some(title) = body.title { album.title = title; }
        if let Some(date) = body.date { album.date = Some(date); }
        services.album_service.update(conn, id, &album)
    })
    .await
    .and_then(|album| {
        AlbumDto::from_album(album)
            .ok_or_else(|| shared::errors::Error::Database("Album has no id".to_string()))
    })
    .map(Json)
    .map_err(|err| crate::utils::error::Error::Custom(CustomError {
        status: Status::InternalServerError,
        code: "Internal".to_string(),
        message: err.to_string(),
    }))
}

#[openapi]
#[delete("/albums/<id>")]
pub async fn delete(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.album_service.delete_by_id(conn, id))
        .await
        .map(|_| Json(Success { success: true }))
        .map_err(|err| crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: err.to_string(),
        }))
}
