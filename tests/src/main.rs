use config::model::AppConfig;

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
    let orchestrator = orchestrator::Orchestrator::new(config);

    let playlist_url = "https://open.spotify.com/playlist/22HjWHbry4q3DzVMOhRqBU?si=ca4f7ddb9afd4ed7";
    let _ = orchestrator.download_playlist_from_url(playlist_url).await.map_err(|e| {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    });

    let track_url = "https://open.spotify.com/track/1HXtMiwXWhWVeBFCovhigZ?si=eb4b20bbc3a64a0c";
    let _ = orchestrator.download_track_from_url(track_url).await.map_err(|e| {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    });
}
