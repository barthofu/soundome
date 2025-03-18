use config::model::AppConfig;
use rsoundcloud::{models::playlist::TrackType, MiscApi, ResourceId, SoundCloudClient, TracksApi};

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

    let client = SoundCloudClient::default().await.unwrap();
    // let track_id = ResourceId::Url(.to_string());

    let track = client.resolve_album_playlist("https://soundcloud.com/bartho-az/sets/euphoria-part-4").await.unwrap();
    println!("{:#?}", track.album_playlist.tracks.iter().map(|t| {
        match t {
            TrackType::Basic(track) => {
                track.track.title.clone()
            },
            TrackType::Mini(track) => {
                track.id.to_string()
            }
        }
    }));
    // let playlist_url = "https://open.spotify.com/playlist/22HjWHbry4q3DzVMOhRqBU?si=ca4f7ddb9afd4ed7";
    // let _ = orchestrator.download_playlist_from_url(playlist_url).await.map_err(|e| {
    //     eprintln!("Error: {:?}", e);
    //     std::process::exit(1);
    // });

    // let track_url = "https://music.youtube.com/watch?v=F1tHJxkEFTI&si=55xfAiLguXtxA0EE";
    // let _ = orchestrator.download_track_from_url(track_url).await.map_err(|e| {
    //     eprintln!("Error: {:?}", e);
    //     std::process::exit(1);
    // });
}
