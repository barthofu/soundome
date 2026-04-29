use std::{path::PathBuf, sync::Arc};

use domain::services::ServiceLayer;
use rocket::{
    form::{Form, FromForm},
    fs::TempFile,
    http::Status,
    post,
    serde::json::Json,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::utils::{database::Db, error::CustomError};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct ImageResponse {
    /// Relative URL of the stored image, e.g. `/images/artists/42.jpg`.
    pub url: String,
}

// ================================================================================================
// Forms
// ================================================================================================

#[derive(FromForm)]
pub struct ImageUpload<'r> {
    pub file: TempFile<'r>,
}

// ================================================================================================
// Helpers
// ================================================================================================

/// Returns the file extension for supported image content types.
fn image_ext(file: &TempFile<'_>) -> Option<&'static str> {
    let ct = file.content_type()?;
    if ct.top().as_str() != "image" {
        return None;
    }
    match ct.sub().as_str() {
        "jpeg" => Some("jpg"),
        "png" => Some("png"),
        "webp" => Some("webp"),
        "gif" => Some("gif"),
        _ => None,
    }
}

/// Removes any previously stored image for `id` under `dir`, across all supported extensions.
fn remove_old_images(dir: &std::path::Path, id: i32) {
    for ext in &["jpg", "jpeg", "png", "webp", "gif"] {
        let _ = std::fs::remove_file(dir.join(format!("{}.{}", id, ext)));
    }
}

/// Saves the uploaded `file` under `data/web/images/<sub_dir>/<id>.<ext>` and returns the
/// public URL path (`/images/<sub_dir>/<id>.<ext>`).
async fn store_image(
    file: &mut TempFile<'_>,
    sub_dir: &str,
    id: i32,
) -> Result<String, crate::utils::error::Error> {
    let ext = image_ext(file).ok_or_else(|| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::UnprocessableEntity,
            code: "UnsupportedMediaType".to_string(),
            message: "Only JPEG, PNG, WebP, and GIF images are accepted".to_string(),
        })
    })?;

    let dir = PathBuf::from("data/web/images").join(sub_dir);
    std::fs::create_dir_all(&dir).map_err(|e| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: format!("Failed to create image directory: {}", e),
        })
    })?;

    remove_old_images(&dir, id);

    let filename = format!("{}.{}", id, ext);
    let dest = dir.join(&filename);

    file.copy_to(&dest).await.map_err(|e| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: format!("Failed to save image file: {}", e),
        })
    })?;

    Ok(format!("/images/{}/{}", sub_dir, filename))
}

// ================================================================================================
// Routes
// ================================================================================================

/// Upload or replace the image for an artist.
///
/// Accepts `multipart/form-data` with a single field named `file`.
/// Supported formats: JPEG, PNG, WebP, GIF.
/// The stored image is served at the URL returned in the response body.
#[post("/artists/<id>/image", data = "<form>")]
pub async fn upload_artist_image(
    id: i32,
    mut form: Form<ImageUpload<'_>>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<ImageResponse>, crate::utils::error::Error> {
    let url = store_image(&mut form.file, "artists", id).await?;
    let image_url = url.clone();
    let services = Arc::clone(services);

    db.run(move |conn| {
        let mut artist = services.artist_service.get_by_id(conn, id)?;
        artist.icon = Some(image_url);
        services.artist_service.update(conn, id, &artist)
    })
    .await
    .map(|_| Json(ImageResponse { url }))
    .map_err(|e| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: e.to_string(),
        })
    })
}

/// Upload or replace the cover image for an album.
///
/// Accepts `multipart/form-data` with a single field named `file`.
/// Supported formats: JPEG, PNG, WebP, GIF.
/// The stored image is served at the URL returned in the response body.
#[post("/albums/<id>/image", data = "<form>")]
pub async fn upload_album_image(
    id: i32,
    mut form: Form<ImageUpload<'_>>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<ImageResponse>, crate::utils::error::Error> {
    let url = store_image(&mut form.file, "albums", id).await?;
    let image_url = url.clone();
    let services = Arc::clone(services);

    db.run(move |conn| {
        let mut album = services.album_service.get_by_id(conn, id)?;
        album.cover = Some(image_url);
        services.album_service.update(conn, id, &album)
    })
    .await
    .map(|_| Json(ImageResponse { url }))
    .map_err(|e| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: e.to_string(),
        })
    })
}

/// Upload or replace the cover image for a track.
///
/// Accepts `multipart/form-data` with a single field named `file`.
/// Supported formats: JPEG, PNG, WebP, GIF.
/// The stored image is served at the URL returned in the response body.
#[post("/tracks/<id>/image", data = "<form>")]
pub async fn upload_track_image(
    id: i32,
    mut form: Form<ImageUpload<'_>>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<ImageResponse>, crate::utils::error::Error> {
    let url = store_image(&mut form.file, "tracks", id).await?;
    let image_url = url.clone();
    let services = Arc::clone(services);

    db.run(move |conn| {
        let mut track = services.track_service.get_by_id(conn, id)?;
        track.cover = Some(image_url);
        services.track_service.update(conn, id, &track)
    })
    .await
    .map(|_| Json(ImageResponse { url }))
    .map_err(|e| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: e.to_string(),
        })
    })
}
