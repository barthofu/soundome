use std::path::Path;

#[tokio::main]
async fn main() {
    orchestrator::workflows::download_spotify_track(
        "https://open.spotify.com/track/1s7rjzZ5cneSISmHt2fqIZ?si=2403e76ff2f34c2b",
        Path::new("/home/bartho/BARTHO/Code/git/soundome/soundome/tmp/library")
    ).await;
}
