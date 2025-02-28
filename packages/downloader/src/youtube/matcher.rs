use shared::models::track::Track;

use super::{Youtube, models::YoutubeSearchResult};

use ngrammatic::{Pad, CorpusBuilder};
use slug::slugify;
use async_trait::async_trait;
use invidious::{hidden::SearchItem, universal::Search, ClientAsyncTrait};

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
        let artist = track
            .artists
            .first() // TODO: really keep only the first artist?
                // because if just join each artist, it can fail like with this track: https://open.spotify.com/track/0qYLUdJQMhrCFA9dNZGcnm?si=509ca53b05c74629
            .map(|artist| artist.name.clone())
            .unwrap_or_default();

        format!("{} {}", artist, track.title)
    }

    async fn get_results(&self, search_query: String) -> Option<Search> {
        self.client
            .search(Some(&format!("q={}&type=video", search_query)))
            .await
            .map_err(|e| {
                eprintln!("Error searching on YouTube: {:?}", e);
            })
            .ok()
    }

    fn apply_pattern(&self, pattern: String, title: String, channel: String) -> String {
        let mut applied_pattern = pattern
            .replace("{{title}}", &title)
            .replace("{{channel}}", &channel)
            .to_lowercase();

        // remove excluded words
        for word in &self.excluded_words {
            applied_pattern = applied_pattern.replace(word, "");
        }

        applied_pattern
    }

    fn match_results(
        &self,
        search_results: Search,
        search_query: String,
        pattern: String,
    ) -> Vec<YoutubeSearchResult> {
        // Create a new text corpus with trigrams (arity = 2) and automatic padding
        let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();
        corpus.add_text(&slugify(&search_query));

        // Iterate through search results and filter videos
        search_results
            .items
            .into_iter()
            .filter_map(|item| {
                if let SearchItem::Video(video) = item {
                    // Apply the pattern to the video's title and author
                    let applied_pattern = self.apply_pattern(pattern.clone(), video.title.clone(), video.author.clone());

                    // Compare the processed title with the search query and calculate a similarity score
                    let result = corpus.search(&applied_pattern, 0.75);

                    // If a match is found, create a YoutubeSearchResult and return it
                    result.first().map(|match_result| {
                        let search_result = YoutubeSearchResult {
                            title: video.title.clone(),
                            duration: video.length as i32,
                            channel: video.author,
                            url: format!("https://www.youtube.com/watch?v={}", video.id),
                            score: match_result.similarity as f32,
                        };

                        println!("[{}] vs [{}] = {}", video.title, search_query, match_result.similarity);
                        search_result
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn order_results(&self, mut results: Vec<YoutubeSearchResult>) -> Vec<YoutubeSearchResult> {
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.to_vec()
    }

    fn get_best_result(&self, ordered_results: Vec<YoutubeSearchResult>, track: Track) -> Option<YoutubeSearchResult> {
        let best_result = ordered_results.first()?;

        if best_result.score < self.treshold {
            return None;
        }

        if let Some(track_duration) = track.duration {
            let offset = track_duration * self.duration_offset_percentage / 100;
            if !(track_duration - offset..=track_duration + offset).contains(&best_result.duration) {
                return None;
            }
        }

        Some(best_result.clone())
    }
}
