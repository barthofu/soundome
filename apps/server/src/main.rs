use std::sync::Arc;

use config::Config;
use database::repositories;
use domain::{ports::repositories::RepositoryLayer, services::ServiceLayer};
use rocket::{catchers, fs::FileServer, launch};
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};

use shared::{init_globals, utils::logs::init_logger};
use soundome_server::utils::database::Db;
use soundome_server::{
    middlewares::cors::Cors,
    routes::{self, errors},
};

fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "../api/openapi.json".to_string(),
        ..Default::default()
    }
}

#[dotenvy::load(path = "./.env", required = true)]
#[launch]
fn rocket() -> _ {

    init_globals().unwrap_or_else(|err| {
        eprintln!("Failed to initialize globals: {}", err);
        std::process::exit(1);
    });

    init_logger();

    tracing::info!("Starting server...");

    let track_repo = Arc::new(repositories::track::DieselTrackRepository::new());
    let album_repo = Arc::new(repositories::album::DieselAlbumRepository::new());
    let artist_repo = Arc::new(repositories::artist::DieselArtistRepository::new());
    let playlist_repo = Arc::new(repositories::playlist::DieselPlaylistRepository::new());
    let task_repo = Arc::new(repositories::task::DieselTaskRepository::new());

    let repositories = Arc::new(RepositoryLayer {
        track: track_repo.clone(),
        album: album_repo.clone(),
        artist: artist_repo.clone(),
        playlist: playlist_repo.clone(),
        task: task_repo.clone(),
    });

    let services = Arc::new(ServiceLayer::new(repositories));

    // Recover tasks that were Running when the server stopped (crash / restart).
    // Reset them to Pending and re-spawn the background jobs.
    {
        let db_url = Config::get().database.url.clone();
        let conn = &mut database::init_connection(&db_url);
        match services.task_service.get_stale_running(conn) {
            Ok(stale_tasks) if !stale_tasks.is_empty() => {
                tracing::warn!("Found {} stale Running task(s) from previous run, re-launching", stale_tasks.len());
                for task in stale_tasks {
                    let task_id = match task.id {
                        Some(id) => id,
                        None => continue,
                    };
                    let url = task.payload.clone();
                    let url = serde_json::from_str::<serde_json::Value>(&url)
                        .ok()
                        .and_then(|v| v.get("url")?.as_str().map(String::from));
                    let Some(url) = url else {
                        tracing::warn!("Task {} has no url in payload, marking as failed", task_id);
                        let _ = services.task_service.set_failed(conn, task_id, "no url in payload");
                        continue;
                    };

                    if let Err(e) = services.task_service.reset_for_retry(conn, task_id) {
                        tracing::error!("Failed to reset task {} for retry: {}", task_id, e);
                        continue;
                    }

                    tracing::info!("Re-launching stale task {} for URL {}", task_id, url);
                    soundome_server::routes::download::spawn_playlist_sync_task(
                        services.clone(),
                        task_id,
                        url,
                    );
                }
            }
            Ok(_) => {} // no stale tasks
            Err(e) => tracing::error!("Failed to check for stale tasks at boot: {}", e),
        }
    }

    // let artist_service = Arc::new(ArtistService::new(artist_repo.clone()));

    rocket::build()
        .attach(Cors)
        .attach(Db::fairing())
        .manage(services)
        .register("/", catchers![errors::default])
        .mount(
            "/api",
            openapi_get_routes![
                routes::misc::index,
                routes::misc::get_all,
                routes::validations::get_pending,
                routes::validations::get_recent,
                routes::validations::approve_validation,
                routes::validations::reject_validation,
                routes::validations::get_match_candidates,
                routes::download::download,
                routes::tasks::get_all,
                routes::tasks::get_by_id,
                routes::tasks::retry,
                routes::tracks::get_all,
                routes::tracks::get,
                routes::tracks::update,
                routes::tracks::delete,
                routes::albums::get_all,
                routes::albums::get,
                routes::albums::update,
                routes::albums::delete,
                routes::artists::get_all,
                routes::artists::get,
                routes::artists::update,
                routes::artists::delete,
            ],
        )
        // .mount("/api", routes![routes::audio::stream,])
        .mount("/swagger", make_swagger_ui(&get_docs()))
        .mount("/", FileServer::from("data/web"))
}
