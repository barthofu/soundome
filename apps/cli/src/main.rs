use config::AppConfig;
use database::get_connection;

#[dotenvy::load(path = "./.env", required = true)]
#[tokio::main]
async fn main() {
    let config = AppConfig::new();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let conn = get_connection(config.database_url.as_str());

    orchestrator::workflows::download_spotify_track(
        "https://open.spotify.com/track/1s7rjzZ5cneSISmHt2fqIZ?si=2403e76ff2f34c2b",
        &conn,
        &config,
    ).await;
}
