use rocket::{get, serde::json::Json};
use rocket_okapi::openapi;
use shared::{models::Artist, types::SoundomeResult};

use crate::utils::database::Db;

#[openapi]
#[get("/")]
pub async fn index() -> Json<String> {
    Json("API is running!".to_owned())
}

// #[openapi]
// #[get("/get-all")]
// pub async fn get_all(
//     db: Db,
// ) -> SoundomeResult<Json<Vec<Artist>>> {
//     db.run(|c| {
//         // Simulate a database query
//         let result = vec!["Item 1".to_string(), "Item 2".to_string()];
//         Ok(result)
//     })
//     .await
//     .map(Json)
//     .map_err(|_| shared::errors::Error::MissingArg)
// }