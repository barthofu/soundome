use std::sync::Arc;

use config::Config;
use domain::services::ServiceLayer;
use rocket::{http::Status, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

use crate::utils::{database::Db, error::CustomError, response::Success};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct ExportResult {
    pub path: String,
}

// ================================================================================================
// Routes
// ================================================================================================

/// Regenerate the M3U8 playlist file for a single playlist.
#[openapi]
#[post("/playlists/<id>/export")]
pub async fn export(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    db.run(move |conn| {
        let playlist = services.playlist_service.get_by_id(conn, id)?;
        let tracks = services.playlist_service.get_tracks(conn, id)?;

        let cfg = Config::get();
        let output_dir = match &cfg.playlists.m3u8_dir {
            Some(dir) => std::path::PathBuf::from(dir),
            None => {
                std::path::PathBuf::from(&cfg.general.base_library_dir).join("Playlists")
            }
        };

        organizer::playlist_writer::write_m3u8(&playlist, &tracks, &output_dir)
            .map_err(|e| shared::errors::Error::Custom(format!("M3U8 export failed: {}", e)))?;

        Ok(())
    })
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
