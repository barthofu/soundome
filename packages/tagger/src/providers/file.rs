use std::path::PathBuf;
use audiotags::{AudioTag, Tag};
use chrono::Datelike;
use shared::{errors::Error, models::track::Track, utils::date::{parse_date, Format}};

use crate::TagWriter;

pub struct File;

impl File {
    pub fn new() -> Self {
        Self
    }
}

impl TagWriter for File {
    fn write(&self, file_path: &PathBuf, track: &Track) -> Result<(), Error> {
        println!("Tagging track: {:?}", track);
        println!("File path: {:?}", file_path);

        let mut tag = Tag::new().read_from_path(file_path)
            .map_err(|e| {
                println!("Error reading tag: {:?}", e);
                Error::InternalServer
            })?;
        convert_track_to_tag(&mut tag, track);
        tag.write_to_path(file_path.display().to_string().as_str())
            .map_err(|e| {
                println!("Error writing tag: {:?}", e);
                Error::InternalServer
            })
    }

}

// ================================================================================================
// Mappers
// ================================================================================================

pub fn convert_track_to_tag(tag: &mut Box<dyn AudioTag + Send + Sync>, track: &Track) {
    tag.set_title(&track.title);
            tag.set_artist(&track.artists.iter()
                .map(|artist| artist.name.as_str())
                .collect::<Vec<&str>>()
                .join(";")
                .as_str());
    track.album.as_ref().map(|album| {
        tag.set_album_title(album.title.as_str());
        tag.set_album_artist(album.artists.iter()
            .map(|artist| artist.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ")
            .as_str());
    });
    track.genre.as_ref().map(|genre| tag.set_genre(genre));
    track.date.as_ref().map(|raw_date|
        parse_date(raw_date, Format::DATE).map(|date|
            tag.set_year(date.year())
        )
    );
    track.track_number.as_ref().map(|track_number| tag.set_track_number(*track_number as u16));
    track.disc_number.as_ref().map(|disc_number| tag.set_disc_number(*disc_number as u16));

    tag.set_comment(
        "Downloaded by Soundome\n---".to_string() +
        &track.url.as_ref().map(|url| "\nSource: ".to_string() + url.as_str()).unwrap_or("".to_string()));
}
