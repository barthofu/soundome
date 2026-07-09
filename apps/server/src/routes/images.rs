use std::{path::PathBuf, sync::Arc};

use domain::services::ServiceLayer;
use rocket::{
    form::{Form, FromForm},
    fs::TempFile,
    http::Status,
    post,
    serde::json::Json,
};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;
use tracing;

use crate::utils::{database::Db, error::CustomError};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct ImageResponse {
    /// Relative URL of the stored image, e.g. `/images/artists/42.jpg`.
    pub url: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct BatchThumbnailResult {
    /// Number of entities that now have an image (fetch succeeded)
    pub count: i32,
    /// Number of entities that have no image after attempting to fetch
    pub skipped: i32,
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

/// Try to resolve an artist's photo from its existing references (Spotify, SoundCloud,
/// YouTube Music) and persist it as the artist's icon.
///
/// Best-effort: returns `404` when none of the artist's references resolve to an
/// image, so the frontend can tell "nothing found" apart from a network/DB error.
#[openapi]
#[post("/artists/<id>/fetch-icon")]
pub async fn fetch_artist_icon(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<ImageResponse>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    let updated = db
        .run(move |conn| {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(
                    services
                        .download_service
                        .fetch_artist_icon_from_references(conn, id),
                )
            })
        })
        .await
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: e.to_string(),
            })
        })?;

    match updated.and_then(|artist| artist.icon) {
        Some(url) => Ok(Json(ImageResponse { url })),
        None => Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::NotFound,
            code: "NoThumbnailFound".to_string(),
            message: "No thumbnail could be resolved from the current references".to_string(),
        })),
    }
}

/// Try to resolve an album's cover from its existing references (Spotify, SoundCloud,
/// YouTube Music) and persist it as the album's cover.
///
/// Best-effort: returns `404` when none of the album's references resolve to an
/// image, so the frontend can tell "nothing found" apart from a network/DB error.
#[openapi]
#[post("/albums/<id>/fetch-cover")]
pub async fn fetch_album_cover(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<ImageResponse>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    let updated = db
        .run(move |conn| {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(
                    services
                        .download_service
                        .fetch_album_cover_from_references(conn, id),
                )
            })
        })
        .await
        .map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: e.to_string(),
            })
        })?;

    match updated.and_then(|album| album.cover) {
        Some(url) => Ok(Json(ImageResponse { url })),
        None => Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::NotFound,
            code: "NoThumbnailFound".to_string(),
            message: "No thumbnail could be resolved from the current references".to_string(),
        })),
    }
}

// ================================================================================================
// Batch thumbnail fetch
// ================================================================================================

#[openapi]
#[post("/batch/fetch-artist-icons")]
pub async fn batch_fetch_artist_icons(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<BatchThumbnailResult>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    let result = db
        .run(move |conn| {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    // Fetch all artists
                    let artists = services.artist_service.get_all(conn).map_err(|e| {
                        crate::utils::error::Error::Custom(CustomError {
                            status: Status::InternalServerError,
                            code: "Internal".to_string(),
                            message: e.to_string(),
                        })
                    })?;

                    let mut count = 0;
                    let mut skipped = 0;

                    // Try to fetch icon for each artist without one
                    for artist in artists {
                        if artist.icon.is_some() {
                            continue;
                        }

                        match services
                            .download_service
                            .fetch_artist_icon_from_references(conn, artist.id.unwrap_or(-1))
                            .await
                        {
                            Ok(Some(_)) => count += 1,
                            Ok(None) => skipped += 1,
                            Err(e) => {
                                tracing::debug!(
                                    "batch_fetch_artist_icons: failed for artist {}: {}",
                                    artist.id.unwrap_or(-1),
                                    e
                                );
                                skipped += 1;
                            }
                        }
                    }

                    Ok::<_, crate::utils::error::Error>(BatchThumbnailResult { count, skipped })
                })
            })
        })
        .await?;

    Ok(Json(result))
}

#[openapi]
#[post("/batch/fetch-album-covers")]
pub async fn batch_fetch_album_covers(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<BatchThumbnailResult>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    let result = db
        .run(move |conn| {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    // Fetch all albums
                    let albums = services.album_service.get_all(conn).map_err(|e| {
                        crate::utils::error::Error::Custom(CustomError {
                            status: Status::InternalServerError,
                            code: "Internal".to_string(),
                            message: e.to_string(),
                        })
                    })?;

                    let mut count = 0;
                    let mut skipped = 0;

                    // Try to fetch cover for each album without one
                    for album in albums {
                        if album.cover.is_some() {
                            continue;
                        }

                        match services
                            .download_service
                            .fetch_album_cover_from_references(conn, album.id.unwrap_or(-1))
                            .await
                        {
                            Ok(Some(_)) => count += 1,
                            Ok(None) => skipped += 1,
                            Err(e) => {
                                tracing::debug!(
                                    "batch_fetch_album_covers: failed for album {}: {}",
                                    album.id.unwrap_or(-1),
                                    e
                                );
                                skipped += 1;
                            }
                        }
                    }

                    Ok::<_, crate::utils::error::Error>(BatchThumbnailResult { count, skipped })
                })
            })
        })
        .await?;

    Ok(Json(result))
}
