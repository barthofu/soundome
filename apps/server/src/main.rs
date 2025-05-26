use std::sync::Arc;

use config::model::AppConfig;
use database::repositories;
use domain::{ports::repositories::RepositoryLayer, services::ServiceLayer};
use rocket::{catchers, launch};
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};

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
    let config = load_config();
    println!("Starting server...");

    let track_repo = Arc::new(repositories::track::DieselTrackRepository::new());
    let album_repo = Arc::new(repositories::album::DieselAlbumRepository::new());
    let artist_repo = Arc::new(repositories::artist::DieselArtistRepository::new());

    let repositories = Arc::new(RepositoryLayer {
        track: track_repo.clone(),
        album: album_repo.clone(),
        artist: artist_repo.clone(),
    });

    let services = Arc::new(ServiceLayer::new(repositories, Arc::new(config)));

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
                // routes::tracks::get,
                // routes::tracks::get_all,
                // routes::tracks::create,
                // routes::tracks::update,
                // routes::tracks::delete,
                // routes::artists::get,
                // routes::artists::get_all,
                // routes::artists::create,
                // routes::artists::update,
                // routes::artists::delete,
                // routes::artists::get_tracks,
                // routes::artists::associate_track,
                // routes::artists::dissociate_track,
            ],
        )
        // .mount("/api", routes![routes::audio::stream,])
        .mount("/swagger", make_swagger_ui(&get_docs()))
}


fn load_config() -> AppConfig {
    println!("Loading configuration...");
    let config = AppConfig::new().unwrap_or_else(|_| {
        eprintln!("Failed to load configuration");
        std::process::exit(1);
    });
    println!("Configuration loaded successfully");
    config
}