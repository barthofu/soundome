use std::path::PathBuf;

use audiotags::Tag;
use config::model::AppConfig;
use fake::{locales::EN, Fake};
use rsoundcloud::models::track::CommentTrack;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tagger::file::get_track_from_file;
use std::time::Instant;

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
    
    let oritingal_file_path = PathBuf::from("/home/coder/library/JEANNINE/Unknown Album/Synaptic Highway.mp3");

    // copy the origin file 1000 times
    use fake::faker::name::raw::*;
    // for i in 1000..1001 {
    //     let new_file_path = PathBuf::from(format!("/home/coder/library/JEANNINE/Unknown Album/Synaptic Highway {}.mp3", i));
    //     std::fs::copy(&oritingal_file_path, &new_file_path).expect("Failed to copy file");

    //     let title = fake::faker::company::en::CompanyName().fake();
    //     let artist_name = Name(EN).fake();
    //     let comment = TrackTagComment {
    //         title: title,
    //         artists: vec![artist_name],
    //     };
    //     set_tag_comment(&new_file_path, &comment);
    //     println!("Copied and tagged file: {:?}", new_file_path);
    // }

    let start = Instant::now();

    let comments = Vec::from_iter((1..=1000).map(|i| {
        let file_path = PathBuf::from(format!("/home/coder/library/JEANNINE/Unknown Album/Synaptic Highway {}.mp3", i));
        get_tag_comment(&file_path).unwrap_or_else(|| TrackTagComment {
            title: format!("Synaptic Highway {}", i),
            artists: vec![Name(EN).fake()],
        })
    }));

    let duration = start.elapsed();
    println!("Time taken to read comments: {:?}", duration);





    // let comments = Vec::new();
    // for i in 1..=1001 {
    //     let file_path = PathBuf::from(format!("/home/coder/library/JEANNINE/Unknown Album/Synaptic Highway {}.mp3", i));
    //     if let Some(comment) = get_tag_comment(&file_path) {
    //         println!("Track {}: {:?}", i, comment);
    //         comments.push(comment);
    //     } else {
    //         println!("Failed to read track from file: {:?}", file_path);
    //     }
    // }


    // for i in 1..10 {
    //     let file_path = PathBuf::from(format!("/home/coder/library/JEANNINE/Unknown Album/Synaptic Highway {}.mp3", i));
    //     if let Some(comment) = get_tag_comment(&file_path) {
    //         println!("Track {}: {:?}", i, comment);
    //     } else {
    //         println!("Failed to read track from file: {:?}", file_path);
    //     }
    // }



    println!("Total comments read: {}", comments.len());

    // let comment = TrackTagComment {
    //     title: "Synaptic Highway".to_string(),
    //     artists: vec!["Jeannine".to_string()],
    // };

    // set_tag_comment(&file_path, &comment);

    // if let Some(comment) = get_tag_comment(&file_path) {
    //     println!("Comment: {:?}", comment);
    // } else {
    //     println!("No comment found in the tag.");
    // }
    
    // let config = AppConfig::new().unwrap();

    // let core = core::Orchestrator::new(config);

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

fn set_tag_comment(file_path: &PathBuf, comment: &TrackTagComment) {
    let comment_str = serde_json::to_string(comment)
        .expect("Failed to serialize comment to JSON");

    let mut tag = Tag::new()
        .read_from_path(&file_path)
        .unwrap();

    tag.set_comment(comment_str);
    tag.write_to_path(file_path.display().to_string().as_str())
        .expect("Failed to write tag to file");
}

fn get_tag_comment(file_path: &PathBuf) -> Option<TrackTagComment> {
    let tag = Tag::new()
        .read_from_path(&file_path)
        .unwrap();

    // println!("Tag: {:#?}", tag.comment());

    tag.comment()
        .and_then(|comment| serde_json::from_str(&comment).ok())
}

// async fn database_test() {
//     let config = AppConfig::new().unwrap();

//     // let db = database::get_connection(&config.database.url);

//     // let track_entity = database::models::track::Track {}
// }
