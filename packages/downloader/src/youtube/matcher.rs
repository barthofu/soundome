use std::{collections::HashMap, f64::consts::E};

use shared::{
    models::Track,
    utils::string::{render_and_normalize_template, string_similarity, SimilarityAlgorithm},
};

use super::Youtube;
use crate::Matcher;
use async_trait::async_trait;

#[async_trait]
impl Matcher for Youtube<'_> {
    fn match_results(&self, search_results: Vec<Track>, source_track: Track) -> Option<Track> {
        let best_match = self
            .patterns
            .iter()
            .flat_map(|pattern| {
                search_results.iter().filter_map(|result| {
                    let video_author = result
                        .artists
                        .first()
                        .map_or("", |artist| artist.name.as_str());
                    let track_artist = source_track.get_primary_artist().name;
                    let track_artists = source_track
                        .artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect::<Vec<String>>()
                        .join(", ");

                    let context = HashMap::from([
                        ("video_title", result.title.as_str()),
                        ("video_author", video_author),
                        ("track_title", &source_track.title),
                        ("track_artist", &track_artist),
                        ("track_artists", &track_artists),
                    ]);

                    let rendered_track_title =
                        render_and_normalize_template(pattern.1, &context, &self.excluded_words).ok()?;
                    let rendered_video_title =
                        render_and_normalize_template(pattern.0, &context, &self.excluded_words).ok()?;

                    let title_score = string_similarity(
                        &rendered_track_title,
                        &rendered_video_title,
                        SimilarityAlgorithm::Smart,
                    ) * 100.0;
                    let duration_score = duration_diff(
                        source_track.duration.unwrap_or(0),
                        result.duration.unwrap_or(0),
                    );

                    println!(
                        "[{}] vs [{}] : title({:.2}) duration({}/{} : {:.2}) ({})",
                        rendered_track_title,
                        rendered_video_title,
                        title_score,
                        source_track.duration.unwrap_or(0),
                        result.duration.unwrap_or(0),
                        duration_score,
                        result
                            .get_provider()
                            .and_then(|provider| provider.external_url)
                            .unwrap_or_default()
                    );

                    if title_score < self.similarity_treshold || duration_score < 25.0 {
                        return None;
                    }
                    if duration_score < 50.0 && title_score < 75.0 {
                        return None;
                    }

                    let similarity_score = if title_score < 0.85 {
                        (title_score + duration_score) / 2.0
                    } else {
                        title_score
                    };

                    Some((similarity_score, result))
                })
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        best_match
            .filter(|(score, _)| *score >= self.similarity_treshold)
            .map(|(_, result)| result.clone())
    }
}

fn duration_diff(song_duration: i32, result_duration: i32) -> f64 {
    let time_diff = (song_duration - result_duration).abs() as f64;
    let score = E.powf(-0.1 * time_diff);
    score * 100.0
}
