use rocket::{get, serde::json::Json};
use rocket_okapi::openapi;

#[openapi]
#[get("/")]
pub async fn index() -> Json<String> {
    Json("API is running!".to_owned())
}
