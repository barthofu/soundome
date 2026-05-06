use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{delete, get, http::Status, patch, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::{Album, Artist, Track};

use crate::utils::{database::Db, error::CustomError, response::Success};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct TrackArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TrackAlbumDto {
    pub id: Option<i32>,
    pub title: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TrackDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<TrackArtistDto>,
    pub album: Option<TrackAlbumDto>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub file_path: Option<String>,
    pub needs_validation: bool,
}

impl TrackDto {
    fn from_track(track: Track) -> Option<Self> {
        Some(Self {
            id: track.id?,
            title: track.title,
            artists: track
                .artists
                .into_iter()
                .map(|a| TrackArtistDto {
                    id: a.id,
                    name: a.name,
                })
                .collect(),
            album: track.album.map(|a| TrackAlbumDto {
                id: a.id,
                title: a.title,
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
            needs_validation: track.needs_validation,
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateTrackBody {
    pub title: Option<String>,
    pub artists: Option<Vec<String>>,
    pub album_title: Option<String>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub cover: Option<String>,
}

// ================================================================================================
// Routes
// ================================================================================================

#[openapi]
#[get("/tracks")]
pub async fn get_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<TrackDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.get_all(conn))
        .await
        .map(|tracks| {
            Json(
                tracks
                    .into_iter()
                    .filter_map(TrackDto::from_track)
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
#[get("/tracks/<id>")]
pub async fn get(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<TrackDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.get_by_id(conn, id))
        .await
        .and_then(|track| {
            TrackDto::from_track(track)
                .ok_or_else(|| shared::errors::Error::Database("Track has no id".to_string()))
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
#[patch("/tracks/<id>", format = "application/json", data = "<body>")]
pub async fn update(
    id: i32,
    body: Json<UpdateTrackBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<TrackDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    db.run(move |conn| {
        let mut track = services.track_service.get_by_id(conn, id)?;

        if let Some(title) = body.title {
            track.title = title;
        }
        if let Some(genre) = body.genre {
            track.genre = Some(genre);
        }
        if let Some(date) = body.date {
            track.date = Some(date);
        }
        if let Some(tn) = body.track_number {
            track.track_number = Some(tn);
        }
        if let Some(dn) = body.disc_number {
            track.disc_number = Some(dn);
        }
        if let Some(label) = body.label {
            track.label = Some(label);
        }
        if let Some(cover) = body.cover {
            track.cover = Some(cover);
        }

        if let Some(names) = body.artists {
            track.artists = names
                .into_iter()
                .map(|name| Artist {
                    id: None,
                    name,
                    icon: None,
                    references: vec![],
                })
                .collect();
        }

        if let Some(album_title) = body.album_title {
            track.album = Some(Album {
                id: track.album.as_ref().and_then(|a| a.id),
                title: album_title,
                artists: track.album.map(|a| a.artists).unwrap_or_default(),
                album_type: shared::models::AlbumType::Album,
                cover: None,
                date: None,
                references: vec![],
            });
        }

        services.track_service.update(conn, id, &track)
    })
    .await
    .and_then(|track| {
        TrackDto::from_track(track)
            .ok_or_else(|| shared::errors::Error::Database("Track has no id".to_string()))
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
#[delete("/tracks/<id>")]
pub async fn delete(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.delete_by_id(conn, id))
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
