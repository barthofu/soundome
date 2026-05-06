use std::sync::Arc;

use diesel::SqliteConnection;
use domain::services::ServiceLayer;

pub async fn domain_tests(services: &Arc<ServiceLayer>, conn: &mut SqliteConnection) {
    download_track(services, conn).await;
}

async fn download_track(services: &Arc<ServiceLayer>, conn: &mut SqliteConnection) {
    tracing::info!("Starting track download test...");

    // Sheng - DIS-MOI PK (remix feat. HONNOJ)
    // let url = "https://open.spotify.com/track/678pEsntcD7rb6eQiy4sKf";
    // let url = "https://soundcloud.com/midori141/sheng-dis-moi-pk";

    // Skone - Générations
    let url = "https://soundcloud.com/skonebzh/generations";
    let res = services
        .download_service
        .download_track_from_url(url, conn)
        .await;
    match res {
        Ok(track) => println!("Downloaded track: {}", track.display()),
        Err(e) => eprintln!("Error downloading track: {:?}", e),
    }

    // tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let url = "https://open.spotify.com/track/0vlYmeolmAWluXfdPZskor?si=6e5b593dd5304b26";
    let res = services
        .download_service
        .download_track_from_url(url, conn)
        .await;
    match res {
        Ok(track) => println!("Downloaded track: {}", track.display()),
        Err(e) => eprintln!("Error downloading track: {:?}", e),
    }

    // let playlist_url = "https://soundcloud.com/barthohm/sets/a-la-derive";

    // let res = services.download_service.sync_playlist_from_url(playlist_url, conn).await;
    // match res {
    //     Ok(_) => println!("Downloaded playlist"),
    //     Err(e) => eprintln!("Error downloading playlist: {:?}", e),
    // }
}
