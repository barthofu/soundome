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
use soundome_server::utils::{
    cancellation::CancellationRegistry, database::Db, task_executor::TaskExecutor,
};
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

    // Resolve (and, if configured, download) the yt-dlp binary to use.
    // This performs network I/O, so it needs a tokio runtime. `#[launch]`'s
    // generated `main()` calls this function synchronously *before* Rocket's
    // own runtime is created (that runtime only starts once we return a
    // built `Rocket<Build>` and `.launch()` is driven), so there is no
    // existing reactor to attach to here. We spin up a short-lived runtime
    // just for this one-off boot task and drop it once done. Never fails:
    // it falls back to "yt-dlp" from PATH on any error (see its doc comment).
    match tokio::runtime::Runtime::new() {
        Ok(rt) => rt.block_on(shared::ytdlp_binary::init()),
        Err(e) => {
            tracing::error!(
                "Failed to create runtime for yt-dlp binary provisioning, falling back to \"yt-dlp\" from PATH: {}",
                e
            );
        }
    }

    // Initialize database and run migrations
    let db_url = Config::get().database.url.clone();
    if let Err(e) = database::init_database(&db_url) {
        tracing::error!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    let track_repo = Arc::new(repositories::track::DieselTrackRepository::new());
    let album_repo = Arc::new(repositories::album::DieselAlbumRepository::new());
    let artist_repo = Arc::new(repositories::artist::DieselArtistRepository::new());
    let playlist_repo = Arc::new(repositories::playlist::DieselPlaylistRepository::new());
    let task_repo = Arc::new(repositories::task::DieselTaskRepository::new());
    let sync_schedule_repo =
        Arc::new(repositories::sync_schedule::DieselSyncScheduleRepository::new());
    let sync_settings_repo =
        Arc::new(repositories::sync_settings::DieselSyncSettingsRepository::new());

    let repositories = Arc::new(RepositoryLayer {
        track: track_repo.clone(),
        album: album_repo.clone(),
        artist: artist_repo.clone(),
        playlist: playlist_repo.clone(),
        task: task_repo.clone(),
        sync_schedule: sync_schedule_repo.clone(),
        sync_settings: sync_settings_repo.clone(),
    });

    let services = Arc::new(ServiceLayer::new(repositories));
    let cancellation_registry = Arc::new(CancellationRegistry::new());
    // Start the serial task executor (single background worker). Every job that
    // needs the shared SQLite DB or long-running network I/O must be enqueued
    // here, so at most one runs at a time. See `utils/task_executor.rs`.
    let task_executor = Arc::new(TaskExecutor::start(
        services.clone(),
        cancellation_registry.clone(),
    ));

    // Automatic recovery of stale tasks (Pending/Running from previous run) is disabled.
    // Stale tasks can be retried manually via the /api/tasks/{id}/retry endpoint or the UI.
    // This ensures operators have full control over task resumption and prevents unexpected
    // behavior after server restarts.
    //
    // To re-enable automatic recovery, uncomment the block below and recompile:
    /*
    {
        let db_url = Config::get().database.url.clone();
        let conn = &mut database::init_connection(&db_url);
        match services.task_service.get_stale_running(conn) {
            Ok(stale_tasks) if !stale_tasks.is_empty() => {
                tracing::warn!(
                    "Found {} stale Running task(s) from previous run, re-enqueueing",
                    stale_tasks.len()
                );
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
                        let _ =
                            services
                                .task_service
                                .set_failed(conn, task_id, "no url in payload");
                        continue;
                    };

                    if let Err(e) = services.task_service.reset_for_retry(conn, task_id) {
                        tracing::error!("Failed to reset task {} for retry: {}", task_id, e);
                        continue;
                    }

                    let cancel_flag = cancellation_registry.register(task_id);
                    tracing::info!("Re-enqueueing stale task {} for URL {}", task_id, url);
                    match task.task_type {
                        shared::models::TaskType::SyncArtist => {
                            task_executor.enqueue_artist_sync(task_id, url, cancel_flag);
                        }
                        shared::models::TaskType::SyncAlbum => {
                            task_executor.enqueue_album_sync(task_id, url, cancel_flag);
                        }
                        _ => {
                            task_executor.enqueue_playlist_sync(task_id, url, cancel_flag);
                        }
                    }
                }
            }
            Ok(_) => {} // no stale tasks
            Err(e) => tracing::error!("Failed to check for stale tasks at boot: {}", e),
        }
    }
    */

    // Spawn the background sync scheduler (checks every 60 seconds).
    // A single global cron (`sync_settings`) decides *when* to run; when due,
    // every enabled subscription (`sync_schedule`) is enqueued in one pass.
    {
        let db_url = Config::get().database.url.clone();
        let services_for_scheduler = services.clone();
        let registry_for_scheduler = cancellation_registry.clone();
        let executor_for_scheduler = task_executor.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(60));

            let conn = &mut database::init_connection(&db_url);

            let settings = match services_for_scheduler.sync_settings_service.get(conn) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Scheduler: failed to load global sync settings: {}", e);
                    continue;
                }
            };

            let now = chrono::Utc::now().naive_utc();
            let is_due = settings.enabled
                && settings
                    .next_run
                    .map(|next_run| next_run <= now)
                    .unwrap_or(true);
            if !is_due {
                continue;
            }

            if let Err(e) = services_for_scheduler.sync_settings_service.mark_ran(conn) {
                tracing::error!("Scheduler: failed to mark global sync settings as ran: {}", e);
                continue;
            }

            let subscriptions = match services_for_scheduler.sync_schedule_service.get_enabled(conn)
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Scheduler: failed to query enabled subscriptions: {}", e);
                    continue;
                }
            };

            for subscription in subscriptions {
                let subscription_id = match subscription.id {
                    Some(id) => id,
                    None => continue,
                };
                let url = subscription.url.clone();
                let label = subscription.label.clone();

                if let Err(e) = services_for_scheduler
                    .sync_schedule_service
                    .mark_ran(conn, subscription_id)
                {
                    tracing::error!(
                        "Scheduler: failed to mark subscription {} as ran: {}",
                        subscription_id,
                        e
                    );
                    continue;
                }

                let task = match subscription.entity_type {
                    shared::models::SyncEntityType::Playlist => services_for_scheduler
                        .task_service
                        .create_playlist_sync(conn, &url, label),
                    shared::models::SyncEntityType::Artist => services_for_scheduler
                        .task_service
                        .create_artist_sync(conn, &url, label),
                };
                let task = match task {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::error!(
                            "Scheduler: failed to create task for subscription {}: {}",
                            subscription_id,
                            e
                        );
                        continue;
                    }
                };
                let task_id = match task.id {
                    Some(id) => id,
                    None => continue,
                };
                let cancel_flag = registry_for_scheduler.register(task_id);
                tracing::info!(
                    "Scheduler: enqueueing sync for subscription {} (type={:?}, url={})",
                    subscription_id,
                    subscription.entity_type,
                    url
                );
                match subscription.entity_type {
                    shared::models::SyncEntityType::Playlist => {
                        executor_for_scheduler.enqueue_playlist_sync(task_id, url, cancel_flag);
                    }
                    shared::models::SyncEntityType::Artist => {
                        executor_for_scheduler.enqueue_artist_sync(task_id, url, cancel_flag);
                    }
                }
            }
        });
    }

    // Rocket — build a figment from the standard Rocket.toml / ROCKET_* sources,
    // then layer any SOUNDOME__SERVER__* overrides on top.
    let figment = {
        let soundome_cfg = Config::get();
        let mut f = rocket::Config::figment();
        // host
        if let Some(host) = &soundome_cfg.server.host {
            f = f.merge(("address", host.as_str()));
        }
        // port
        if let Some(port) = soundome_cfg.server.port {
            f = f.merge(("port", port));
        }
        // rocket database
        // let db: rocket::figment::value::Map<_, rocket::figment::value::Value>  = rocket::figment::util::map! {
        //     "url" => soundome_cfg.database.url.as_str().into(),
        //     "pool_size" => 10.into(),
        //     "timeout" => 5.into(),
        // };
        // f = f.merge(("databases.sqlite", db));
        f = f.merge(("databases.sqlite.url", soundome_cfg.database.url.as_str()));

        f
    };

    rocket::custom(figment)
        .attach(Cors)
        .attach(Db::fairing())
        .manage(services)
        .manage(cancellation_registry)
        .manage(task_executor)
        .register("/", catchers![errors::default])
        .mount(
            "/api",
            openapi_get_routes![
                routes::misc::index,
                routes::misc::get_all,
                routes::misc::get_providers,
                routes::misc::get_version,
                routes::validations::get_pending,
                routes::validations::get_recent,
                routes::validations::approve_validation,
                routes::validations::reject_validation,
                routes::validations::get_match_candidates,
                routes::validations::get_youtube_provider_candidates,
                routes::download::download,
                routes::tasks::get_all,
                routes::tasks::get_by_id,
                routes::tasks::retry,
                routes::tasks::cancel,
                routes::sync_schedules::get_all,
                routes::sync_schedules::get_by_id,
                routes::sync_schedules::create,
                routes::sync_schedules::update,
                routes::sync_schedules::delete,
                routes::sync_schedules::trigger,
                routes::sync_settings::get_settings,
                routes::sync_settings::update_settings,
                routes::sync_settings::trigger_all,
                routes::tracks::get_all,
                routes::tracks::get,
                routes::tracks::update,
                routes::tracks::delete,
                routes::tracks::download_file,
                routes::tracks::get_references,
                routes::tracks::add_reference,
                routes::tracks::delete_reference,
                routes::albums::get_all,
                routes::albums::get,
                routes::albums::update,
                routes::albums::delete,
                routes::albums::merge,
                routes::albums::get_references,
                routes::albums::add_reference,
                routes::albums::delete_reference,
                routes::images::fetch_album_cover,
                routes::artists::get_all,
                routes::artists::get,
                routes::artists::update,
                routes::artists::delete,
                routes::artists::merge,
                routes::artists::get_references,
                routes::artists::add_reference,
                routes::artists::delete_reference,
                routes::images::fetch_artist_icon,
                routes::playlists::get_all,
                routes::playlists::get_tracks,
                routes::playlists::export,
                routes::playlists::delete,
                routes::library::scan,
                routes::library::ingest,
                routes::library::list_ingest_files,
                routes::library::ingest_all,
                routes::storage::storage_stats,
            ],
        )
        // .mount("/api", routes![routes::audio::stream,])
        .mount(
            "/api",
            routes![
                routes::images::upload_artist_image,
                routes::images::upload_album_image,
                routes::images::upload_track_image,
                routes::images::batch_fetch_artist_icons,
                routes::images::batch_fetch_album_covers,
            ],
        )
        .mount("/", routes![routes::metrics::metrics])
        .mount("/swagger", make_swagger_ui(&get_docs()))
        .mount("/", FileServer::from("data/web"))
}
