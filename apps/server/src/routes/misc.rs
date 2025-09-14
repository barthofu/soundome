use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{get, http::Status, serde::json::Json};
use rocket_okapi::openapi;
use shared::models::Artist;

use crate::utils::{database::Db, error::CustomError};

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
        .map_err(|err|
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        )
}