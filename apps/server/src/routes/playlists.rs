use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{get, http::Status, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

use crate::utils::{database::Db, error::CustomError, response::Success};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct PlaylistDto {
    pub id: i32,
    pub name: String,
    pub source: String,
    pub source_url: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PlaylistTrackArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PlaylistTrackAlbumDto {
    pub id: Option<i32>,
    pub title: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PlaylistTrackDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<PlaylistTrackArtistDto>,
    pub album: Option<PlaylistTrackAlbumDto>,
    pub duration: Option<i32>,
    pub cover: Option<String>,
    pub genre: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ExportResult {
    pub path: String,
}

// ================================================================================================
// Routes
// ================================================================================================

/// List all playlists in the library.
#[openapi]
#[get("/playlists")]
pub async fn get_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<PlaylistDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    db.run(move |conn| services.playlist_service.get_all(conn))
        .await
        .map(|playlists| {
            Json(
                playlists
                    .into_iter()
                    .filter_map(|p| {
                        Some(PlaylistDto {
                            id: p.id?,
                            name: p.name,
                            source: p.source.as_ref().to_string(),
                            source_url: p.source_url,
                            cover: p.cover,
                        })
                    })
                    .collect(),
            )
        })
        .map_err(|err: shared::errors::Error| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

/// Get all tracks for a playlist.
#[openapi]
#[get("/playlists/<id>/tracks")]
pub async fn get_tracks(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<PlaylistTrackDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    db.run(move |conn| services.playlist_service.get_tracks(conn, id))
        .await
        .map(|tracks| {
            Json(
                tracks
                    .into_iter()
                    .filter_map(|t| {
                        Some(PlaylistTrackDto {
                            id: t.id?,
                            title: t.title,
                            artists: t
                                .artists
                                .into_iter()
                                .map(|a| PlaylistTrackArtistDto {
                                    id: a.id,
                                    name: a.name,
                                })
                                .collect(),
                            album: t.album.map(|a| PlaylistTrackAlbumDto {
                                id: a.id,
                                title: a.title,
                            }),
                            duration: t.duration,
                            cover: t.cover,
                            genre: t.genre,
                        })
                    })
                    .collect(),
            )
        })
        .map_err(|err: shared::errors::Error| match err {
            shared::errors::Error::NotFound(_) => crate::utils::error::Error::Custom(CustomError {
                status: Status::NotFound,
                code: "NotFound".to_string(),
                message: err.to_string(),
            }),
            _ => crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            }),
        })
}

/// Regenerate the M3U8 playlist file for a single playlist.
#[openapi]
#[post("/playlists/<id>/export")]
pub async fn export(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    db.run(move |conn| services.playlist_service.export_m3u8(conn, id))
        .await
        .map(|_| Json(Success { success: true }))
        .map_err(|err: shared::errors::Error| match err {
            shared::errors::Error::Database(_) => crate::utils::error::Error::Custom(CustomError {
                status: Status::NotFound,
                code: "NotFound".to_string(),
                message: err.to_string(),
            }),
            _ => crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            }),
        })
}
