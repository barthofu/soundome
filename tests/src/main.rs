use std::sync::Arc;

use config::model::AppConfig;
use database::repositories;
use ::domain::{ports::repositories::RepositoryLayer, services::ServiceLayer};

use serde::{Deserialize, Serialize};

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

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
       
    let config = AppConfig::new().unwrap();

    let track_repo = Arc::new(repositories::track::DieselTrackRepository::new());
    let album_repo = Arc::new(repositories::album::DieselAlbumRepository::new());
    let artist_repo = Arc::new(repositories::artist::DieselArtistRepository::new());

    let repositories = Arc::new(RepositoryLayer {
        track: track_repo.clone(),
        album: album_repo.clone(),
        artist: artist_repo.clone(),
    });

    let conn = &mut database::init_connection(&config.database.url);

    let services = Arc::new(ServiceLayer::new(repositories, Arc::new(config)));

    domain::domain_tests(&services, conn).await;
}