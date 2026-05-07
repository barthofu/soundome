use audiotags::{AudioTag, Tag};
use id3::TagLike;
use shared::{
    errors::Error,
    models::{Album, Artist, Track},
    types::SoundomeResult,
};
use std::{path::PathBuf, str::FromStr};

// ================================================================================================
// SOUNDOME_ID custom-tag constants
// ================================================================================================

const SOUNDOME_ID_KEY: &str = "SOUNDOME_ID";
const MP4_MEAN: &str = "com.soundome";
const MP4_NAME: &str = "ID";

// ================================================================================================
// Public API
// ================================================================================================

/**
 * Reads the tag from a file and returns a converted Track object.
 */
pub fn get_track_from_file(file_path: &PathBuf) -> SoundomeResult<Track> {
    tracing::info!("Reading tag from file: {:?}", file_path);

    let mut track = Tag::new()
        .read_from_path(file_path)
        .map(|tag| convert_tag_to_track(&*tag))
        .map_err(|e| Error::Custom(format!("Error reading audio tags: {:?}", e)))?;

    // Best-effort: read the SOUNDOME_ID custom tag
    track.soundome_id = read_soundome_id_from_file(file_path);

    Ok(track)
}

/**
 * Tag an audio file with the provided track information.
 * Also writes the SOUNDOME_ID custom tag when `track.soundome_id` is set.
 */
pub fn tag_file_with_track(file_path: &PathBuf, track: &Track) -> SoundomeResult<()> {
    let mut tag = Tag::new()
        .read_from_path(file_path)
        .map_err(|e| Error::Custom(format!("Error reading audio tags: {:?}", e)))?;
    convert_track_to_tag(&mut tag, track);
    tag.write_to_path(file_path.display().to_string().as_str())
        .map_err(|e| Error::Custom(format!("Error writing audio tags: {:?}", e)))?;

    // Write the SOUNDOME_ID custom tag if present
    if let Some(ref sid) = track.soundome_id {
        write_soundome_id_tag(file_path, sid)?;
    }

    Ok(())
}

/// Write `SOUNDOME_ID` as a custom tag frame into the file.
///
/// | Format    | Tag frame              |
/// |-----------|------------------------|
/// | MP3 / ID3 | `TXXX:SOUNDOME_ID`     |
/// | FLAC      | Vorbis comment         |
/// | MP4 / M4A | `----:com.soundome:ID` |
pub fn write_soundome_id_tag(file_path: &PathBuf, soundome_id: &str) -> SoundomeResult<()> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "mp3" => write_soundome_id_id3(file_path, soundome_id),
        "flac" => write_soundome_id_flac(file_path, soundome_id),
        "m4a" | "mp4" | "aac" => write_soundome_id_mp4(file_path, soundome_id),
        // For unknown / unsupported formats log a warning and continue.
        other => {
            tracing::warn!(
                "Cannot write SOUNDOME_ID: unsupported extension {:?} for {:?}",
                other,
                file_path
            );
            Ok(())
        }
    }
}

/// Read `SOUNDOME_ID` from the custom tag of an audio file.
/// Returns `None` if the tag is absent or the format is unsupported.
pub fn read_soundome_id_from_file(file_path: &PathBuf) -> Option<String> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())?;

    match ext.as_str() {
        "mp3" => read_soundome_id_id3(file_path),
        "flac" => read_soundome_id_flac(file_path),
        "m4a" | "mp4" | "aac" => read_soundome_id_mp4(file_path),
        _ => None,
    }
}

// ================================================================================================
// Format-specific helpers
// ================================================================================================

fn write_soundome_id_id3(file_path: &PathBuf, soundome_id: &str) -> SoundomeResult<()> {
    let mut tag = id3::Tag::read_from_path(file_path).unwrap_or_default();

    // Remove any existing SOUNDOME_ID TXXX frame to avoid duplicates.
    tag.remove_extended_text(Some(SOUNDOME_ID_KEY), None);

    tag.add_frame(id3::frame::ExtendedText {
        description: SOUNDOME_ID_KEY.to_string(),
        value: soundome_id.to_string(),
    });

    tag.write_to_path(file_path, id3::Version::Id3v24)
        .map_err(|e| Error::Custom(format!("Failed to write ID3 TXXX frame: {}", e)))
}

fn read_soundome_id_id3(file_path: &PathBuf) -> Option<String> {
    let tag = id3::Tag::read_from_path(file_path).ok()?;
    let value = tag
        .extended_texts()
        .find(|t| t.description == SOUNDOME_ID_KEY)
        .map(|t| t.value.clone());
    value
}

fn write_soundome_id_flac(file_path: &PathBuf, soundome_id: &str) -> SoundomeResult<()> {
    let mut tag = metaflac::Tag::read_from_path(file_path)
        .map_err(|e| Error::Custom(format!("Failed to read FLAC tags: {}", e)))?;

    let comments = tag.vorbis_comments_mut();
    // Replace any existing entry
    comments.remove(SOUNDOME_ID_KEY);
    comments.set(SOUNDOME_ID_KEY, vec![soundome_id.to_string()]);

    tag.write_to_path(file_path)
        .map_err(|e| Error::Custom(format!("Failed to write FLAC Vorbis comment: {}", e)))
}

fn read_soundome_id_flac(file_path: &PathBuf) -> Option<String> {
    let tag = metaflac::Tag::read_from_path(file_path).ok()?;
    tag.vorbis_comments()
        .and_then(|vc| vc.get(SOUNDOME_ID_KEY))
        .and_then(|v| v.first())
        .cloned()
}

fn write_soundome_id_mp4(file_path: &PathBuf, soundome_id: &str) -> SoundomeResult<()> {
    let mut tag = mp4ameta::Tag::read_from_path(file_path)
        .map_err(|e| Error::Custom(format!("Failed to read MP4 tags: {}", e)))?;

    let fourcc = mp4ameta::FreeformIdent::new(MP4_MEAN, MP4_NAME);
    tag.remove_data_of(&fourcc);
    tag.set_data(
        fourcc,
        mp4ameta::Data::Utf8(soundome_id.to_string()),
    );

    tag.write_to_path(file_path)
        .map_err(|e| Error::Custom(format!("Failed to write MP4 freeform atom: {}", e)))
}

fn read_soundome_id_mp4(file_path: &PathBuf) -> Option<String> {
    let tag = mp4ameta::Tag::read_from_path(file_path).ok()?;
    let fourcc = mp4ameta::FreeformIdent::new(MP4_MEAN, MP4_NAME);
    let value = tag.strings_of(&fourcc).next().map(|s| s.to_string());
    value
}

// ================================================================================================
// Mappers
// ================================================================================================

fn convert_track_to_tag(tag: &mut Box<dyn AudioTag + Send + Sync>, track: &Track) {
    tag.set_title(&track.title);
    tag.set_artist(
        track
            .artists
            .iter()
            .map(|artist| artist.name.as_str())
            .collect::<Vec<&str>>()
            .join(";")
            .as_str(),
    );
    if let Some(album) = track.album.as_ref() {
        tag.set_album_title(album.title.as_str());
        tag.set_album_artist(
            album
                .artists
                .iter()
                .map(|artist| artist.name.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
                .as_str(),
        );
    }
    if let Some(genre) = track.genre.as_ref() {
        tag.set_genre(genre);
    }
    if let Some(date) = track.date.as_ref() {
        tag.set_date(id3::Timestamp::from_str(date).unwrap_or(id3::Timestamp::default()))
    }
    if let Some(track_number) = track.track_number.as_ref() {
        tag.set_track_number(*track_number as u16);
    }
    if let Some(disc_number) = track.disc_number.as_ref() {
        tag.set_disc_number(*disc_number as u16);
    }
    // tag.album_cover()

    tag.set_comment(
        "Downloaded by Soundome\n---".to_string(), // + "\nSource: "
                                                   // + track
                                                   //     .source
                                                   //     .as_ref()
                                                   //     .unwrap_or(&TrackSource::Unknown)
                                                   //     .as_ref()
                                                   // + "\nProvider: "
                                                   // + track
                                                   //     .provider
                                                   //     .as_ref()
                                                   //     .unwrap_or(&TrackProvider::Unknown)
                                                   //     .as_ref(),
    );
}

fn convert_tag_to_track(tag: &(dyn AudioTag + Send + Sync)) -> Track {
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
        id: None,
        needs_validation: false,
        validation_reason: None,
        soundome_id: None,
        title: tag
            .title()
            .map_or("Unknown".to_string(), |title| title.to_string()),
        artists: tag
            .artists()
            .map(|artists| {
                artists
                    .iter()
                    .map(|artist| Artist {
                        id: None,
                        name: artist.to_string(),
                        icon: None,
                        references: Vec::new(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        album: tag.album_title().map(|album_title| Album {
            id: None,
            title: album_title.to_string(),
            artists: tag
                .album_artist()
                .map(|artist| {
                    artist
                        .split(";")
                        .map(|artist| Artist {
                            id: None,
                            name: artist.to_string(),
                            icon: None,
                            references: Vec::new(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            album_type: shared::models::AlbumType::Unknown,
            date: date.clone(),
            cover: None,
            references: Vec::new(),
        }),
        genre: tag.genre().map(|genre| genre.to_string()),
        date,
        cover: None, // TODO
        disc_number: tag.disc_number().map(|disc_number| disc_number as i32),
        track_number: tag.track_number().map(|track_number| track_number as i32),
        duration: tag.duration().map(|duration| duration as i32),
        label: None,
        file_path: None,
        references: Vec::new(),
    }
}

