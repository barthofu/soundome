use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{http::Status, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::{database::Db, error::CustomError};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DownloadRequest {
    pub url: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DownloadResult {
    pub title: String,
    pub artists: Vec<String>,
    pub needs_validation: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PlaylistDownloadResult {
    pub downloaded: usize,
    pub skipped: usize,
    pub failed: usize,
}

// ================================================================================================
// Helpers
// ================================================================================================

fn is_playlist_url(url: &str) -> bool {
    url.contains("/playlist/")
        || url.contains("/sets/")
        || url.contains("list=")
        || url.contains("/album/")
}

// ================================================================================================
// Routes
// ================================================================================================

#[openapi]
#[post("/download", format = "json", data = "<body>")]
pub async fn download(
    body: Json<DownloadRequest>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<serde_json::Value>, crate::utils::error::Error> {
    let url = body.into_inner().url;
    let services = Arc::clone(services);

    if is_playlist_url(&url) {
        let tracks = db
            .run(move |conn| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(services.download_service.sync_playlist_from_url(&url, conn))
                })
            })
            .await
            .map_err(|err| {
                crate::utils::error::Error::Custom(CustomError {
                    status: Status::InternalServerError,
                    code: "DownloadFailed".to_string(),
                    message: err.to_string(),
                })
            })?;

        let downloaded = tracks.len();
        Ok(Json(serde_json::json!({
            "type": "playlist",
            "downloaded": downloaded,
        })))
    } else {
        let track = db
            .run(move |conn| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(services.download_service.download_track_from_url(&url, conn))
                })
            })
            .await
            .map_err(|err| {
                crate::utils::error::Error::Custom(CustomError {
                    status: Status::InternalServerError,
                    code: "DownloadFailed".to_string(),
                    message: err.to_string(),
                })
            })?;

        Ok(Json(serde_json::json!({
            "type": "track",
            "title": track.title,
            "artists": track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
            "needs_validation": track.needs_validation,
        })))
    }
}
