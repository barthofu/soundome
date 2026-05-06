use std::sync::Arc;

use domain::services::ServiceLayer;
use rocket::{get, http::ContentType};

use crate::utils::database::Db;

/// Prometheus metrics endpoint — returns library statistics as Prometheus text format.
/// Not registered with OpenAPI; scrape at `GET /metrics`.
#[get("/metrics")]
pub async fn metrics(db: Db, services: &rocket::State<Arc<ServiceLayer>>) -> (ContentType, String) {
    let services = Arc::clone(services);

    let body = db
        .run(move |conn| {
            let tracks = services.track_service.count(conn).unwrap_or(0);
            let tracks_pending = services
                .track_service
                .count_pending_validations(conn)
                .unwrap_or(0);
            let albums = services.album_service.count(conn).unwrap_or(0);
            let artists = services.artist_service.count(conn).unwrap_or(0);
            let playlists = services.playlist_service.count(conn).unwrap_or(0);
            let tasks_pending = services
                .task_service
                .count_by_status(conn, "Pending")
                .unwrap_or(0);
            let tasks_running = services
                .task_service
                .count_by_status(conn, "Running")
                .unwrap_or(0);
            let tasks_completed = services
                .task_service
                .count_by_status(conn, "Completed")
                .unwrap_or(0);
            let tasks_failed = services
                .task_service
                .count_by_status(conn, "Failed")
                .unwrap_or(0);
            let tasks_cancelled = services
                .task_service
                .count_by_status(conn, "Cancelled")
                .unwrap_or(0);

            format!(
                "\
# HELP soundome_tracks_total Total number of tracks in the library
# TYPE soundome_tracks_total gauge
soundome_tracks_total {tracks}
# HELP soundome_tracks_pending_validation Number of tracks awaiting manual validation
# TYPE soundome_tracks_pending_validation gauge
soundome_tracks_pending_validation {tracks_pending}
# HELP soundome_albums_total Total number of albums in the library
# TYPE soundome_albums_total gauge
soundome_albums_total {albums}
# HELP soundome_artists_total Total number of artists in the library
# TYPE soundome_artists_total gauge
soundome_artists_total {artists}
# HELP soundome_playlists_total Total number of playlists tracked
# TYPE soundome_playlists_total gauge
soundome_playlists_total {playlists}
# HELP soundome_tasks_total Number of tasks by status
# TYPE soundome_tasks_total gauge
soundome_tasks_total{{status=\"Pending\"}} {tasks_pending}
soundome_tasks_total{{status=\"Running\"}} {tasks_running}
soundome_tasks_total{{status=\"Completed\"}} {tasks_completed}
soundome_tasks_total{{status=\"Failed\"}} {tasks_failed}
soundome_tasks_total{{status=\"Cancelled\"}} {tasks_cancelled}
"
            )
        })
        .await;

    (
        ContentType::new("text", "plain").with_params([("version", "0.0.4"), ("charset", "utf-8")]),
        body,
    )
}
