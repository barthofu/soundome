use std::path::PathBuf;

use config::AppConfig;
use database::get_connection;
use fetcher::{spotify::Spotify, Fetcher};
use shared::{errors::Error, utils::{enums::Match, string::string_similarity}};
use tagger::{file::get_track_from_file, TagProvider};

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
    // let conn = get_connection(config.database_url.as_str());


    // let track = get_track_from_file(&PathBuf::from("tmp/library/NecromatiK.mp3"));
    // println!("Track: {:#?}", track.unwrap());

    // let url = "https://open.spotify.com/track/5gT5lvU5TyTnsPO4bCUOaH?si=8c52751de3bd4e12";


    // let spotify = Spotify::new(&config.spotify.client_id, &config.spotify.client_secret)
    //     .map_err(|_| Error::Config).unwrap();

    // let track = Spotify::get_track_from_url(&spotify, url).map_err(|_| Error::NotFound).unwrap();
    // let track2 = Spotify::get_track_from_url(&spotify, "https://open.spotify.com/track/0nzP00NGbnCnDAvl3AS5BT?si=6851feac97af45f9").map_err(|_| Error::NotFound).unwrap();
    // // println!("Track: {:#?}", track);
    // let matching_score_tmp = track.compare(&track2);
    // println!("Matching score: {}", matching_score_tmp);


    let musicbrainz_provider = tagger::providers::musicbrainz::MusicBrainz::new();
    let track = musicbrainz_provider.get_match_from_query("Baleines zinée").await;
    match track {
        Match::Exact(track) => {
            println!("Exact match: {:#?}", track);
        },
        Match::Partial(track) => {
            println!("Partial match: {:#?}", track);
        },
        Match::None => {
            println!("No match found");
        }
    }
    // println!("Results: {:#?}", results.iter().map(|track| track.title.clone()).collect::<Vec<String>>());
    // let result = musicbrainz_provider.get(&track).await;
    // println!("Results: {:?}", results);

    // println!("Matching score: {}", matching_score);
}
