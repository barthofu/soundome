#[derive(Debug, Clone)]
pub struct YoutubeSearchResult {
    pub title: String,
    pub duration: i32,
    pub channel: String,
    pub url: String,
    pub similarity_score: f64
}

impl YoutubeSearchResult {

    pub fn new(
        title: String,
        duration: i32,
        channel: String,
        id: String,
        score: f64
    ) -> Self {
        Self {
            title,
            duration,
            channel,
            url: format!("https://www.youtube.com/watch?v={}", id),
            similarity_score: score
        }
    }
}