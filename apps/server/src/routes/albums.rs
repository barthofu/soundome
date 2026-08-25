use std::sync::Arc;

use domain::ports::repositories::{AlbumQuery, AlbumSortBy, SortDir};
use domain::services::ServiceLayer;
use rocket::{delete, get, http::Status, patch, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::Album;

use crate::routes::tracks::{reference_to_dto, AddReferenceBody, ReferenceDto, TrackDto};
use crate::utils::{
    database::Db,
    error::CustomError,
    response::{
        is_desc, normalize_search, resolve_offset, resolve_page, resolve_page_size, PageDto,
        Success,
    },
};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct AlbumArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AlbumDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<AlbumArtistDto>,
    pub album_type: String,
    pub cover: Option<String>,
    pub date: Option<String>,
    pub references: Vec<ReferenceDto>,
}

impl AlbumDto {
    pub(crate) fn from_album(album: Album) -> Option<Self> {
        Some(Self {
            id: album.id?,
            title: album.title,
            artists: album
                .artists
                .into_iter()
                .map(|a| AlbumArtistDto {
                    id: a.id,
                    name: a.name,
                })
                .collect(),
            album_type: album.album_type.as_ref().to_string(),
            cover: album.cover,
            date: album.date,
            references: album.references.into_iter().map(reference_to_dto).collect(),
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAlbumBody {
    pub title: Option<String>,
    pub date: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MergeAlbumsBody {
    /// Album IDs to merge into `target_id`. These will be deleted after the merge.
    pub source_ids: Vec<i32>,
    /// The album to keep.
    pub target_id: i32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct NamePairDto {
    pub id: i32,
    pub title: String,
}

// ================================================================================================
// Routes
// ================================================================================================

#[openapi]
#[get("/albums?<page>&<page_size>&<q>&<sort_by>&<sort_dir>")]
pub async fn get_all(
    page: Option<i64>,
    page_size: Option<i64>,
    q: Option<String>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<PageDto<AlbumDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    let sort_by = match sort_by.as_deref() {
        Some("date") => AlbumSortBy::Date,
        Some("artist") => AlbumSortBy::Artist,
        Some("track_count") => AlbumSortBy::TrackCount,
        _ => AlbumSortBy::Title,
    };
    let sort_dir = if is_desc(&sort_dir) {
        SortDir::Desc
    } else {
        SortDir::Asc
    };
    let page = resolve_page(page);
    let page_size = resolve_page_size(page_size);
    let query = AlbumQuery {
        offset: resolve_offset(page, page_size),
        limit: page_size,
        search: normalize_search(&q),
        sort_by,
        sort_dir,
    };

    db.run(move |conn| services.album_service.get_page(conn, query))
        .await
        .map(|result| {
            Json(PageDto {
                items: result
                    .items
                    .into_iter()
                    .filter_map(AlbumDto::from_album)
                    .collect(),
                total: result.total,
                page,
                page_size,
            })
        })
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

/// Lightweight `(id, title)` pairs for every album — used by the "find similar"
/// duplicate-detection workflow, which needs the full library's identity but
/// none of its heavy fields (covers, references, ...).
#[openapi]
#[get("/albums/names")]
pub async fn get_names(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<NamePairDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.album_service.get_names(conn))
        .await
        .map(|pairs| {
            Json(
                pairs
                    .into_iter()
                    .map(|p| NamePairDto {
                        id: p.id,
                        title: p.name,
                    })
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

/// All tracks belonging to an album (album drill-down view).
#[openapi]
#[get("/albums/<id>/tracks")]
pub async fn get_tracks(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<TrackDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.get_by_album(conn, id))
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
#[get("/albums/<id>")]
pub async fn get(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<AlbumDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.album_service.get_by_id(conn, id))
        .await
        .and_then(|album| {
            AlbumDto::from_album(album)
                .ok_or_else(|| shared::errors::Error::Database("Album has no id".to_string()))
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
#[patch("/albums/<id>", format = "application/json", data = "<body>")]
pub async fn update(
    id: i32,
    body: Json<UpdateAlbumBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<AlbumDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    db.run(move |conn| {
        let mut album = services.album_service.get_by_id(conn, id)?;
        if let Some(title) = body.title {
            album.title = title;
        }
        if let Some(date) = body.date {
            album.date = Some(date);
        }
        if let Some(cover) = body.cover {
            album.cover = Some(cover);
        }
        services.album_service.update(conn, id, &album)
    })
    .await
    .and_then(|album| {
        AlbumDto::from_album(album)
            .ok_or_else(|| shared::errors::Error::Database("Album has no id".to_string()))
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
#[delete("/albums/<id>")]
pub async fn delete(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.album_service.delete_by_id(conn, id))
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

#[openapi]
#[post("/albums/merge", format = "application/json", data = "<body>")]
pub async fn merge(
    body: Json<MergeAlbumsBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<AlbumDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    if body.source_ids.is_empty() {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "source_ids must not be empty".to_string(),
        }));
    }
    if body.source_ids.contains(&body.target_id) {
        return Err(crate::utils::error::Error::Custom(CustomError {
            status: Status::BadRequest,
            code: "BadRequest".to_string(),
            message: "target_id must not appear in source_ids".to_string(),
        }));
    }

    db.run(move |conn| {
        services
            .album_service
            .merge_into(conn, &body.source_ids, body.target_id)
    })
    .await
    .and_then(|album| {
        AlbumDto::from_album(album)
            .ok_or_else(|| shared::errors::Error::Database("Album has no id".to_string()))
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

// ================================================================================================
// Reference sub-resource
// ================================================================================================

/// List all references attached to an album.
#[openapi]
#[get("/albums/<id>/references")]
pub async fn get_references(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<ReferenceDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.album_service.get_by_id(conn, id))
        .await
        .map(|album| Json(album.references.into_iter().map(reference_to_dto).collect()))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::NotFound,
                code: "NotFound".to_string(),
                message: err.to_string(),
            })
        })
}

/// Add a reference to an album.
#[openapi]
#[post(
    "/albums/<id>/references",
    format = "application/json",
    data = "<body>"
)]
pub async fn add_reference(
    id: i32,
    body: Json<AddReferenceBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<ReferenceDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let reference = body.into_inner().into_reference();

    db.run(move |conn| services.album_service.add_reference(conn, id, reference))
        .await
        .map(|refs| Json(refs.into_iter().map(reference_to_dto).collect()))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: err.to_string(),
            })
        })
}

/// Remove a single reference from an album by its reference row ID.
#[openapi]
#[delete("/albums/<_id>/references/<ref_id>")]
pub async fn delete_reference(
    _id: i32,
    ref_id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.album_service.delete_reference(conn, ref_id))
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
