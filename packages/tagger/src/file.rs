use std::{path::PathBuf, str::FromStr};
use audiotags::{AudioTag, Tag};
use shared::{errors::Error, models::{album::Album, artist::Artist, track::{Track, TrackProvider, TrackSource}}};

/**
 * Reads the tag from a file and returns a converted Track object.
 */
pub fn get_track_from_file(file_path: &PathBuf) -> Result<Track, Error> {
    println!("Reading tag from file: {:?}", file_path);

    Tag::new().read_from_path(file_path)
        .map(|tag| convert_tag_to_track(&tag))
        .map_err(|e| Error::Custom(format!("Error reading audio tags: {:?}", e)))
}

/**
 * Tag an audio file with the provided track information.
 */
pub fn tag_file_with_track(file_path: &PathBuf, track: &Track) -> Result<(), Error> {
    let mut tag = Tag::new().read_from_path(file_path)
        .map_err(|e| Error::Custom(format!("Error reading audio tags: {:?}", e)))?;
    convert_track_to_tag(&mut tag, track);
    tag.write_to_path(file_path.display().to_string().as_str())
        .map_err(|e| Error::Custom(format!("Error writing audio tags: {:?}", e)))
}

// ================================================================================================
// Mappers
// ================================================================================================

fn convert_track_to_tag(tag: &mut Box<dyn AudioTag + Send + Sync>, track: &Track) {
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
    track.date.as_ref().map(|date| tag.set_date(id3::Timestamp::from_str(&date).unwrap_or(id3::Timestamp::default())));
    track.track_number.as_ref().map(|track_number| tag.set_track_number(*track_number as u16));
    track.disc_number.as_ref().map(|disc_number| tag.set_disc_number(*disc_number as u16));
    // tag.album_cover()

    tag.set_comment(
        "Downloaded by Soundome\n---".to_string() +
        "\nSource: " + track.source.as_ref().unwrap_or(&TrackSource::Unknown).as_ref() +
        "\nProvider: " + track.provider.as_ref().unwrap_or(&TrackProvider::Unknown).as_ref()
    );
}

fn convert_tag_to_track(tag: &Box<dyn AudioTag + Send + Sync>) -> Track {
    let date = tag.date().map(|date| {
        let mut date_str = format!("{:04}", date.year);
        if let Some(month) = date.month {
            date_str += &format!("-{:02}", month);
            if let Some(day) = date.day {
                date_str += &format!("-{:02}", day);
            }
        }
        date_str
    });

    Track {
        title: tag.title().map_or("Unknown".to_string(), |title| title.to_string()),
        artists: tag.artists().map(|artists| artists
            .iter()
            .map(|artist| Artist {
                name: artist.to_string(),
                url: None,
                icon: None
            }).collect()).unwrap_or_default(),
        album: tag.album_title().map(|album_title| Album {
            title: album_title.to_string(),
            artists: tag.album_artist().map(|artist| artist.split(";")
                .map(|artist| Artist {
                    name: artist.to_string(),
                    url: None,
                    icon: None
                }).collect()).unwrap_or_default(),
            date: date.clone(),
            url: None,
            cover: None
        }),
        genre: tag.genre().map(|genre| genre.to_string()),
        date: date,
        cover: None, // TODO
        disc_number: tag.disc_number().map(|disc_number| disc_number as i32),
        track_number: tag.track_number().map(|track_number| track_number as i32),
        duration: tag.duration().map(|duration| duration as i32),
        label: None,
        file_path: None,
        source: None,
        source_url: None,
        provider: None,
        provider_url: None
    }
}
