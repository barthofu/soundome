use std::sync::Arc;

use domain::services::ServiceLayer;
use diesel::SqliteConnection;

pub async fn domain_tests(services: &Arc<ServiceLayer>, conn: &mut SqliteConnection) {
    download_track(services, conn).await;
    
}

async fn download_track(services: &Arc<ServiceLayer>, conn: &mut SqliteConnection) {

    let url = "https://open.spotify.com/track/678pEsntcD7rb6eQiy4sKf";

    let res = services.download_service.download_track_from_url(url, conn).await;
    match res {
        Ok(track) => println!("Downloaded track: {}", track.display()),
        Err(e) => eprintln!("Error downloading track: {:?}", e),
    }


    // // let playlist_url = "https://open.spotify.com/playlist/22HjWHbry4q3DzVMOhRqBU?si=ca4f7ddb9afd4ed7";
    // let playlist_url = "https://soundcloud.com/bartho-az/sets/euphoria-part-4";
    // let _ = core.download_playlist_from_url(playlist_url).await.map_err(|e| {
    //     eprintln!("Error: {:?}", e);
    //     std::process::exit(1);
    // });

    // let track_url = "https://soundcloud.com/jeannindamix/mamakkat-jeannine-synaptic-highway";
    // let _ = core.download_track_from_url(track_url).await.map_err(|e| {
    //     eprintln!("Error: {:?}", e);
    //     std::process::exit(1);
    // });
}