use std::sync::Arc;

use config::Config;
use domain::services::ServiceLayer;
use rocket::{get, http::Status, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;
use shared::models::Artist;

use crate::utils::{database::Db, error::CustomError};

#[derive(Serialize, JsonSchema)]
pub struct VersionResponse {
    /// Application version (from Cargo.toml at compile time).
    pub version: &'static str,
}

/// Returns the current server version.
#[openapi]
#[get("/version")]
pub fn get_version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[openapi]
#[get("/")]
pub async fn index() -> Json<String> {
    Json("API is running!".to_owned())
}

#[openapi]
#[get("/get-all")]
pub async fn get_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<Artist>>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    db.run(move |conn| services.artist_service.get_all(conn))
        .await
        .map(Json)
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

#[derive(Serialize, JsonSchema)]
pub struct ProvidersResponse {
    /// Source providers that are enabled and fully configured.
    pub providers: Vec<String>,
}

/// Returns the list of source providers that are available (enabled + configured).
#[openapi]
#[get("/providers")]
pub fn get_providers() -> Json<ProvidersResponse> {
    let config = Config::get();
    let mut providers = Vec::new();

    // Spotify requires non-empty credentials
    if let Some(spotify) = config.providers.spotify.as_ref() {
        if !spotify.client_id.is_empty() && !spotify.client_secret.is_empty() {
            providers.push("Spotify".to_string());
        }
    }

    // SoundCloud and YouTube don't require credentials
    providers.push("SoundCloud".to_string());
    providers.push("YouTube".to_string());
    providers.push("YouTube Music".to_string());

    Json(ProvidersResponse { providers })
}
