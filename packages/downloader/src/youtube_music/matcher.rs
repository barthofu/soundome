use super::YoutubeMusic;
use crate::Matcher;
use async_trait::async_trait;
use shared::models::track::Track;

#[async_trait]
impl Matcher for YoutubeMusic {
    fn match_results(&self, search_results: Vec<Track>, source_track: Track) -> Option<String> {
        let best_match = search_results
            .iter()
            .map(|result| (source_track.compare(result), result.clone()))
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        best_match
            .filter(|(score, _)| *score >= self.similarity_treshold)
            .map(|(_, result)| result.provider_url.clone())
            .flatten()
    }
}
