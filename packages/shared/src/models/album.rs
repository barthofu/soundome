use super::artist::Artist;

#[derive(Debug, Clone)]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
    EP,
    Mixtape,
    Soundtrack,
    Live,
    Remix,
    Bootleg,
    DJMix,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub title: String,
    pub artists: Vec<Artist>,
    pub album_type: AlbumType,
    pub url: Option<String>,
    pub cover: Option<String>,
    pub date: Option<String>,
}
