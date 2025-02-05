use shared::models::track::Track;

use super::{Youtube, models::YoutubeSearchResult};

use ngrammatic::{Pad, CorpusBuilder};
use slug::slugify;
use async_trait::async_trait;
use invidious::{hidden::SearchItem, universal::Search, ClientAsync, ClientAsyncTrait};

#[async_trait]
pub trait Matcher {

    /// create a search query from a track
    fn create_search_query(&self, track: Track) -> String;

    /// search youtube for a track
    async fn get_results(&self, search_query: String) -> Option<Search>;

    // apply a pattern to a title and a channel
    fn apply_pattern(&self, pattern: String, title: String, channel: String) -> String;

    /// match results with a search query
    fn match_results(&self, search_results: Search, search_query: String, pattern: String) -> Vec<YoutubeSearchResult>;

    /// order results by score
    fn order_results(&self, results: Vec<YoutubeSearchResult>) -> Vec<YoutubeSearchResult>;

    /// get the best result
    fn get_best_result(&self, ordered_results: Vec<YoutubeSearchResult>, track: Track) -> Option<YoutubeSearchResult>;
}

#[async_trait]
impl Matcher for Youtube {

    fn create_search_query(&self, track: Track) -> String {

        let mut search_query = String::new();

        // artists
        for artist in track.artists {
            search_query.push_str(&artist.name);
            search_query.push_str(" ");
        }

        // song
        search_query.push_str(&track.title);

        search_query
    }

    async fn get_results(&self, search_query: String) -> Option<Search> {

        let client = ClientAsync::default();
        let search_results = client
            .search(Some(format!("q={}&type=video", search_query).as_str()))
            .await;

        match search_results {
            Ok(results) => Some(results),
            Err(_) => None,
        }
    }

    fn apply_pattern(&self, pattern: String, title: String, channel: String) -> String {

        let mut applied_pattern = pattern
            .replace("{{title}}", title.as_str())
            .replace("{{channel}}", channel.as_str())
            .to_lowercase();

        // remove excluded words
        for word in &self.excluded_words {
            applied_pattern = applied_pattern.replace(word.as_str(), "");
        }

        applied_pattern
    }

    fn match_results(&self, search_results: Search, search_query: String, pattern: String) -> Vec<YoutubeSearchResult> {

        let mut weighted_results: Vec<YoutubeSearchResult> = vec![];

        let mut corpus = CorpusBuilder::new()
            .arity(2)
            .pad_full(Pad::Auto)
            .finish();

        corpus.add_text(slugify(search_query.clone()).as_str());

        for item in search_results.items {
            if let SearchItem::Video(video) = item {

                // apply pattern to video infos
                let applied_pattern = self.apply_pattern(pattern.clone(), video.title.clone(), video.author.clone());

                // compare the title with the track and get a matching score
                let result = corpus.search(&applied_pattern, 0.75);

                if result.first().is_some() {
                    let youtube_search_result = YoutubeSearchResult {
                        title: video.title.clone(),
                        duration: video.length as i32,
                        channel: video.author,
                        url: format!("https://www.youtube.com/watch?v={}", video.id),
                        score: result.first().unwrap().similarity as f32,
                    };
                    weighted_results.push(youtube_search_result);
                    println!("[{}] vs [{}] = {}", video.title.clone(), search_query, result.first().unwrap().similarity);
                }
            }
        };

        weighted_results

    }

    /// order results by score
    fn order_results(&self, mut results: Vec<YoutubeSearchResult>) -> Vec<YoutubeSearchResult> {

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.to_vec()
    }

    fn get_best_result(&self, ordered_results: Vec<YoutubeSearchResult>, track: Track) -> Option<YoutubeSearchResult> {

        let best_result = ordered_results.first();

        // guards
        if best_result.is_none() { return None; }
        else if best_result.unwrap().score < self.treshold { return None; }
        else if track.duration.is_some() {

            let offset = track.duration.unwrap() * self.duration_offset_percentage / 100;

            if best_result.unwrap().duration > track.duration.unwrap() + offset { return None; }
            else if best_result.unwrap().duration < track.duration.unwrap() - offset { return None; }
        }

        best_result.map(|r| r.clone())
    }
}
