use std::sync::Arc;

use config::Config;
use database::repositories;
use domain::{ports::repositories::RepositoryLayer, services::ServiceLayer};
use rocket::{catchers, fs::FileServer, launch, routes};
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

#[dotenvy::load(path = "./.env", required = false)]
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
    let sync_schedule_repo = Arc::new(repositories::sync_schedule::DieselSyncScheduleRepository::new());

    let repositories = Arc::new(RepositoryLayer {
        track: track_repo.clone(),
        album: album_repo.clone(),
        artist: artist_repo.clone(),
        playlist: playlist_repo.clone(),
        task: task_repo.clone(),
        sync_schedule: sync_schedule_repo.clone(),
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

    // Spawn the background sync scheduler (checks every 60 seconds)
    {
        let db_url = Config::get().database.url.clone();
        let services_for_scheduler = services.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));

                let conn = &mut database::init_connection(&db_url);
                let due = match services_for_scheduler.sync_schedule_service.get_due(conn) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("Scheduler: failed to query due schedules: {}", e);
                        continue;
                    }
                };
                for schedule in due {
                    let schedule_id = match schedule.id {
                        Some(id) => id,
                        None => continue,
                    };
                    let url = schedule.playlist_url.clone();
                    let label = schedule.label.clone();
                    if let Err(e) = services_for_scheduler.sync_schedule_service.mark_ran(conn, schedule_id) {
                        tracing::error!("Scheduler: failed to mark schedule {} as ran: {}", schedule_id, e);
                        continue;
                    }
                    let task = match services_for_scheduler.task_service.create_playlist_sync(conn, &url, label) {
                        Ok(t) => t,
                        Err(e) => {
                            tracing::error!("Scheduler: failed to create task for schedule {}: {}", schedule_id, e);
                            continue;
                        }
                    };
                    let task_id = match task.id {
                        Some(id) => id,
                        None => continue,
                    };
                    tracing::info!("Scheduler: triggering sync for schedule {} (url={})", schedule_id, url);
                    soundome_server::routes::download::spawn_playlist_sync_task(
                        services_for_scheduler.clone(),
                        task_id,
                        url,
                    );
                }
            }
        });
    }

    // let artist_service = Arc::new(ArtistService::new(artist_repo.clone()));

    // Rocket — build a figment from the standard Rocket.toml / ROCKET_* sources,
    // then layer any SOUNDOME__SERVER__* overrides on top.
    let figment = {
        let soundome_cfg = Config::get();
        let mut f = rocket::Config::figment();
        if let Some(host) = &soundome_cfg.server.host {
            f = f.merge(("address", host.as_str()));
        }
        if let Some(port) = soundome_cfg.server.port {
            f = f.merge(("port", port));
        }
        f
    };

    rocket::custom(figment)
        .attach(Cors)
        .attach(Db::fairing())
        .manage(services)
        .register("/", catchers![errors::default])
        .mount(
            "/api",
            openapi_get_routes![
                routes::misc::index,
                routes::misc::get_all,
                routes::misc::get_providers,
                routes::validations::get_pending,
                routes::validations::get_recent,
                routes::validations::approve_validation,
                routes::validations::reject_validation,
                routes::validations::get_match_candidates,
                routes::download::download,
                routes::tasks::get_all,
                routes::tasks::get_by_id,
                routes::tasks::retry,
                routes::sync_schedules::get_all,
                routes::sync_schedules::get_by_id,
                routes::sync_schedules::create,
                routes::sync_schedules::update,
                routes::sync_schedules::delete,
                routes::sync_schedules::trigger,
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
                routes::artists::merge,
            ],
        )
        // .mount("/api", routes![routes::audio::stream,])
        .mount("/api", routes![
            routes::images::upload_artist_image,
            routes::images::upload_album_image,
            routes::images::upload_track_image,
        ])
        .mount("/swagger", make_swagger_ui(&get_docs()))
        .mount("/", FileServer::from("data/web"))
}
