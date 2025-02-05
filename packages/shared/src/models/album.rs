use super::artist::Artist;

#[derive(Debug, Clone)]
pub struct Album {
    pub title: String,
    pub url: Option<String>,
    pub cover: Option<String>,
    pub date: Option<String>,
    pub artists: Vec<Artist>,
}