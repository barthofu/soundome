use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{get, http::Status, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

use crate::utils::{database::Db, error::CustomError};

// ================================================================================================
// DTOs
// ================================================================================================

/// Storage usage for a single artist
#[derive(Debug, Serialize, JsonSchema)]
pub struct ArtistStorageDto {
    pub id: i32,
    pub name: String,
    pub bytes: u64,
    pub percent: f64,
}

/// Overall library storage statistics
#[derive(Debug, Serialize, JsonSchema)]
pub struct StorageStatsDto {
    /// Total library size in bytes
    pub total_bytes: u64,
    /// Total library size formatted (e.g., "12.5 GB")
    pub total_formatted: String,
    /// Per-artist breakdown, sorted by size descending
    pub artists: Vec<ArtistStorageDto>,
}

// ================================================================================================
// Routes
// ================================================================================================

/// Get library storage statistics.
///
/// Returns total library size and a breakdown by artist, sorted by size descending.
/// Useful for understanding which artists consume the most disk space.
#[openapi(tag = "library")]
#[get("/library/storage-stats")]
pub async fn storage_stats(
    db: Db,
    services: &rocket::State<Arc<ServiceLayer>>,
) -> Result<Json<StorageStatsDto>, crate::utils::error::Error> {
    let services = Arc::clone(services);

    db.run(move |conn| {
        // Get all artists
        let artists = services.artist_service.get_all(conn).map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: format!("Failed to get artists: {}", e),
            })
        })?;

        // Get all tracks with file paths
        let tracks = services.track_service.get_all(conn).map_err(|e| {
            crate::utils::error::Error::Custom(CustomError {
                status: Status::InternalServerError,
                code: "Internal".to_string(),
                message: format!("Failed to get tracks: {}", e),
            })
        })?;

        // Calculate total bytes and per-artist breakdown
        let mut artist_bytes: std::collections::HashMap<i32, u64> =
            std::collections::HashMap::new();
        let mut total_bytes: u64 = 0;

        for track in &tracks {
            if let Some(file_path) = &track.file_path {
                // file_path is stored as a cwd-relative path (e.g. "./library/Artist/Album/track.mp3").
                // Use it directly; Path::join would corrupt it by prepending base_library_dir.
                if let Ok(metadata) = std::fs::metadata(file_path) {
                    let file_bytes = metadata.len();
                    total_bytes += file_bytes;

                    // Attribute bytes to each artist on the track
                    for artist in &track.artists {
                        if let Some(artist_id) = artist.id {
                            *artist_bytes.entry(artist_id).or_insert(0) += file_bytes;
                        }
                    }
                }
            }
        }

        // Build artist storage DTOs
        let mut artist_dtos: Vec<ArtistStorageDto> = artists
            .into_iter()
            .filter_map(|artist| {
                let id = artist.id?;
                let bytes = *artist_bytes.get(&id).unwrap_or(&0);
                Some(ArtistStorageDto {
                    id,
                    name: artist.name,
                    bytes,
                    percent: if total_bytes > 0 {
                        (bytes as f64 / total_bytes as f64) * 100.0
                    } else {
                        0.0
                    },
                })
            })
            .collect();

        // Sort by bytes descending
        artist_dtos.sort_by_key(|a| std::cmp::Reverse(a.bytes));

        let total_formatted = format_bytes(total_bytes);

        Ok(Json(StorageStatsDto {
            total_bytes,
            total_formatted,
            artists: artist_dtos,
        }))
    })
    .await
}

// ================================================================================================
// Helpers
// ================================================================================================

/// Format bytes into human-readable format (B, KB, MB, GB, TB)
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    match unit_idx {
        0 => format!("{} {}", bytes, UNITS[0]),
        _ => format!("{:.1} {}", size, UNITS[unit_idx]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
        assert_eq!(format_bytes(1_099_511_627_776), "1.0 TB");
        assert_eq!(format_bytes(5_368_709_120), "5.0 GB");
    }
}
