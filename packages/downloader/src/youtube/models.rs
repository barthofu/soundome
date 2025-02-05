#[derive(Debug, Clone)]
pub struct YoutubeSearchResult {
    pub title: String,
    pub duration: i32,
    pub channel: String,
    pub url: String,
    pub score: f32
}

impl YoutubeSearchResult {

    pub fn new(
        title: String,
        duration: i32,
        channel: String,
        id: String,
        score: f32
    ) -> Self {
        Self {
            title,
            duration,
            channel,
            url: format!("https://www.youtube.com/watch?v={}", id),
            score
        }
    }
}