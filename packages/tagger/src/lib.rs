use std::path::PathBuf;
use audiotags::Tag;
use shared::{errors::Error, models::track::Track};

mod mappers;

pub fn tag_track(file_path: &PathBuf, track: &Track) -> Result<(), Error> {
    println!("Tagging track: {:?}", track);
    println!("File path: {:?}", file_path);

    let mut tag = Tag::new().read_from_path(file_path)
        .map_err(|e| {
            println!("Error reading tag: {:?}", e);
            Error::InternalServer
        })?;
    mappers::convert_track_to_tag(&mut tag, track);
    tag.write_to_path(file_path.display().to_string().as_str())
        .map_err(|e| {
            println!("Error writing tag: {:?}", e);
            Error::InternalServer
        })
}
