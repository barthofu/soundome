use std::sync::Arc;

use diesel::SqliteConnection;
use domain::services::ServiceLayer;

pub async fn tmp_tests(services: &Arc<ServiceLayer>, conn: &mut SqliteConnection) {
    // let url = "https://soundcloud.com/skonebzh/generations";
    // let url = "https://soundcloud.com/sousacps/back-to-origin";
    // let url = "https://soundcloud.com/barthohm/sets/mentalcore";
    // let res = services
    //     .download_service
    //     .sync_playlist_from_url(url, conn)
    //     .await;
    // match res {
    //     Ok(track) => println!("Downloaded track: {}", track.display()),
    //     Err(e) => eprintln!("Error downloading track: {:?}", e),
    // }
}
