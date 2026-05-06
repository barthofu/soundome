use std::sync::Arc;

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
