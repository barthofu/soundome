use config::AppConfig;
use fetcher::{spotify::Spotify, Fetcher};
use shared::{errors::Error, utils::string::string_similarity};
use tagger::TagProvider;

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

    let url = "https://open.spotify.com/track/5gT5lvU5TyTnsPO4bCUOaH?si=8c52751de3bd4e12";

    string_similarity("testtestest", "testtesttest");

    // let conn = get_connection(config.database_url.as_str());

    let spotify = Spotify::new(&config.spotify.client_id, &config.spotify.client_secret)
        .map_err(|_| Error::Config).unwrap();

    let track = Spotify::get_track_from_url(&spotify, url).map_err(|_| Error::NotFound).unwrap();
    let track2 = Spotify::get_track_from_url(&spotify, "https://open.spotify.com/track/0nzP00NGbnCnDAvl3AS5BT?si=6851feac97af45f9").map_err(|_| Error::NotFound).unwrap();
    // println!("Track: {:#?}", track);
    let matching_score_tmp = track.compare(&track, &track2);
    println!("Matching score: {}", matching_score_tmp);


    let musicbrainz_provider = tagger::providers::musicbrainz::MusicBrainz::new();
    let results = musicbrainz_provider.search(&track).await;
    // println!("Results: {:?}", results);

    let matching_score = track.compare(&track, &results.first().unwrap());
    println!("Matching score: {}", matching_score);
}
