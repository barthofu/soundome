use std::sync::Arc;

use ::domain::{ports::repositories::RepositoryLayer, services::ServiceLayer};
use config::Config;
use database::repositories;

use serde::{Deserialize, Serialize};
use shared::{init_globals, utils::logs::init_logger};

mod domain;
mod file;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackTagComment {
    title: String,
    artists: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct Track {
    title: String,
    artists: Vec<String>,
}

#[dotenvy::load(path = "./.env", required = true)]
#[tokio::main]
async fn main() {
    println!("Starting tests...");

    init_globals().unwrap_or_else(|err| {
        eprintln!("Failed to initialize globals: {}", err);
        std::process::exit(1);
    });

    init_logger();

    let track_repo = Arc::new(repositories::track::DieselTrackRepository::new());
    let album_repo = Arc::new(repositories::album::DieselAlbumRepository::new());
    let artist_repo = Arc::new(repositories::artist::DieselArtistRepository::new());
    let playlist_repo = Arc::new(repositories::playlist::DieselPlaylistRepository::new());
    let task_repo = Arc::new(repositories::task::DieselTaskRepository::new());
    let sync_schedule_repo =
        Arc::new(repositories::sync_schedule::DieselSyncScheduleRepository::new());

    let repositories = Arc::new(RepositoryLayer {
        track: track_repo.clone(),
        album: album_repo.clone(),
        artist: artist_repo.clone(),
        playlist: playlist_repo.clone(),
        task: task_repo.clone(),
        sync_schedule: sync_schedule_repo.clone(),
    });

    let conn = &mut database::init_connection(&Config::get().database.url);

    let services = Arc::new(ServiceLayer::new(repositories));

    domain::domain_tests(&services, conn).await;
}
