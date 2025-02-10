use audiotags::AudioTag;
use chrono::{Datelike, NaiveDate};
use shared::models::track::Track;

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
        NaiveDate::parse_from_str(raw_date.as_str(), "%Y-%m-%d").map(|date|
            tag.set_year(date.year())
        )
    );
    track.track_number.as_ref().map(|track_number| tag.set_track_number(*track_number as u16));
    track.disc_number.as_ref().map(|disc_number| tag.set_disc_number(*disc_number as u16));

    tag.set_comment(
        "Downloaded by Soundome\n---".to_string() +
        &track.url.as_ref().map(|url| "\nSource: ".to_string() + url.as_str()).unwrap_or("".to_string()));
}
