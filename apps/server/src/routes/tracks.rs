use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::fs::NamedFile;
use rocket::{delete, get, http::Status, patch, post, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::models::{Album, Artist, Platform, Reference, ReferenceType, Track};

use crate::utils::{database::Db, error::CustomError, response::Success};

// ================================================================================================
// DTOs
// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct TrackArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TrackAlbumDto {
    pub id: Option<i32>,
    pub title: String,
}

// ================================================================================================
// Shared reference DTO (used by all entity routes)
// ================================================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReferenceDto {
    pub id: Option<i32>,
    pub ref_type: String,
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

pub fn reference_to_dto(r: Reference) -> ReferenceDto {
    ReferenceDto {
        id: r.id,
        ref_type: r.ref_type.as_ref().to_string(),
        platform: r.platform.as_ref().to_string(),
        external_id: r.external_id,
        external_url: r.external_url,
    }
}

/// Body for manually adding a reference to any entity.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddReferenceBody {
    /// One of: Source, Provider, Metadata, Reference
    pub ref_type: String,
    /// One of: Spotify, SoundCloud, MusicBrainz, YoutubeMusic, Youtube, Bandcamp, Unknown
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

impl AddReferenceBody {
    pub fn into_reference(self) -> Reference {
        Reference {
            id: None,
            ref_type: ReferenceType::from_str(&self.ref_type),
            platform: Platform::from_str(&self.platform),
            external_id: self.external_id,
            external_url: self.external_url,
        }
    }
}

// ================================================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct TrackDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<TrackArtistDto>,
    pub album: Option<TrackAlbumDto>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub file_path: Option<String>,
    pub needs_validation: bool,
    pub references: Vec<ReferenceDto>,
}

impl TrackDto {
    fn from_track(track: Track) -> Option<Self> {
        Some(Self {
            id: track.id?,
            title: track.title,
            artists: track
                .artists
                .into_iter()
                .map(|a| TrackArtistDto {
                    id: a.id,
                    name: a.name,
                })
                .collect(),
            album: track.album.map(|a| TrackAlbumDto {
                id: a.id,
                title: a.title,
            }),
            date: track.date,
            genre: track.genre,
            cover: track.cover,
            duration: track.duration,
            track_number: track.track_number,
            disc_number: track.disc_number,
            label: track.label,
            file_path: track
                .file_path
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            needs_validation: track.needs_validation,
            references: track.references.into_iter().map(reference_to_dto).collect(),
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateTrackBody {
    pub title: Option<String>,
    pub artists: Option<Vec<String>>,
    pub album_title: Option<String>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub cover: Option<String>,
}

// ================================================================================================
// Routes
// ================================================================================================

#[openapi]
#[get("/tracks")]
pub async fn get_all(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<TrackDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.get_all(conn))
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
#[get("/tracks/<id>")]
pub async fn get(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<TrackDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.get_by_id(conn, id))
        .await
        .and_then(|track| {
            TrackDto::from_track(track)
                .ok_or_else(|| shared::errors::Error::Database("Track has no id".to_string()))
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
#[patch("/tracks/<id>", format = "application/json", data = "<body>")]
pub async fn update(
    id: i32,
    body: Json<UpdateTrackBody>,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<TrackDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    let body = body.into_inner();

    db.run(move |conn| -> shared::types::SoundomeResult<Track> {
        let old_track = services.track_service.get_by_id(conn, id)?;
        let mut track = old_track.clone();

        if let Some(title) = body.title {
            track.title = title;
        }
        if let Some(genre) = body.genre {
            track.genre = Some(genre);
        }
        if let Some(date) = body.date {
            track.date = Some(date);
        }
        if let Some(tn) = body.track_number {
            track.track_number = Some(tn);
        }
        if let Some(dn) = body.disc_number {
            track.disc_number = Some(dn);
        }
        if let Some(label) = body.label {
            track.label = Some(label);
        }
        if let Some(cover) = body.cover {
            track.cover = Some(cover);
        }

        if let Some(names) = body.artists {
            // Deduplicate artist names and reuse or create Artist records
            let mut artists = Vec::new();
            for name in names {
                let artist = Artist {
                    id: None,
                    name,
                    icon: None,
                    references: vec![],
                };
                let saved = services.artist_service.create_or_ignore(conn, &artist)?;
                artists.push(saved);
            }
            track.artists = artists;
        }

        if let Some(album_title) = body.album_title {
            // Preserve album ID and other metadata, updating the title
            let existing_album = track.album.clone();
            let new_album = Album {
                id: existing_album.as_ref().and_then(|a| a.id),
                title: album_title.clone(),
                artists: existing_album.as_ref().map(|a| a.artists.clone()).unwrap_or_default(),
                album_type: shared::models::AlbumType::Album,
                cover: existing_album.as_ref().and_then(|a| a.cover.clone()),
                date: existing_album.as_ref().and_then(|a| a.date.clone()),
                references: existing_album.as_ref().map(|a| a.references.clone()).unwrap_or_default(),
            };
            
            // If album has an ID, update it; otherwise, create_or_ignore will handle it in create_or_update
            if let Some(album_id) = new_album.id {
                if let Err(e) = services.album_service.update(conn, album_id, &new_album) {
                    tracing::warn!("Failed to update album: {}", e);
                }
            }
            
            track.album = Some(new_album);
        }

        // Update file if it exists and metadata changed
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = services
                    .download_service
                    .update_track_file_metadata(&old_track, &mut track)
                    .await
                {
                    tracing::warn!("Failed to update track file metadata: {}", e);
                    // Don't fail the entire request if file update fails
                }
            })
        });

        services.track_service.create_or_update(conn, &track)
    })
    .await
    .and_then(|track| {
        TrackDto::from_track(track)
            .ok_or_else(|| shared::errors::Error::Database("Track has no id".to_string()))
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
#[delete("/tracks/<id>")]
pub async fn delete(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.delete_by_id(conn, id))
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

/// Download the audio file for a track.
#[openapi]
#[get("/tracks/<id>/download")]
pub async fn download_file(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<NamedFile, crate::utils::error::Error> {
    let services = Arc::clone(services);

    let track = db
        .run(move |conn| services.track_service.get_by_id(conn, id))
        .await
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: match err {
                    shared::errors::Error::NotFound(_) => Status::NotFound,
                    _ => Status::InternalServerError,
                },
                code: "NotFound".to_string(),
                message: err.to_string(),
            })
        })?;

    let file_path = track.file_path.ok_or_else(|| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::NotFound,
            code: "NoFile".to_string(),
            message: "Track has no local file".to_string(),
        })
    })?;

    NamedFile::open(&file_path).await.map_err(|_| {
        crate::utils::error::Error::Custom(CustomError {
            status: Status::NotFound,
            code: "FileNotFound".to_string(),
            message: format!("Audio file not found on disk: {}", file_path.display()),
        })
    })
}

// ================================================================================================
// Reference sub-resource
// ================================================================================================

/// List all references attached to a track.
#[openapi]
#[get("/tracks/<id>/references")]
pub async fn get_references(
    id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Vec<ReferenceDto>>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.get_by_id(conn, id))
        .await
        .map(|track| Json(track.references.into_iter().map(reference_to_dto).collect()))
        .map_err(|err| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::NotFound,
                code: "NotFound".to_string(),
                message: err.to_string(),
            })
        })
}

/// Add a reference to a track.
#[openapi]
#[post(
    "/tracks/<id>/references",
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

    db.run(move |conn| services.track_service.add_reference(conn, id, reference))
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

/// Remove a single reference from a track by its reference row ID.
#[openapi]
#[delete("/tracks/<_id>/references/<ref_id>")]
pub async fn delete_reference(
    _id: i32,
    ref_id: i32,
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<Success>, crate::utils::error::Error> {
    let services = Arc::clone(services);
    db.run(move |conn| services.track_service.delete_reference(conn, ref_id))
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
