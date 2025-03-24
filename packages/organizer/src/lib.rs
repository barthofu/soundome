use std::path::PathBuf;

use config::model::AppConfig;
use shared::{errors::Error, models::track::Track, types::SoundomeResult};

pub fn move_track_file(track: &mut Track, base_dir: &str) -> SoundomeResult<()> {

    println!("Moving track file: {:?}", track.file_path);

    let file_path = track
        .file_path
        .as_ref()
        .ok_or(Error::Custom("Track file path is missing".to_string()))?;
    let file_name = file_path.file_name().ok_or(Error::Custom("Invalid file path".to_string()))?;

    let artist_folder_name = track
        .artists
        .first()
        .map(|artist| artist.name.clone())
        .unwrap_or("Unknown Artist".to_string());

    let album_folder_name = track
        .album
        .as_ref()
        .map(|album| album.title.clone())
        .unwrap_or("Unknown Album".to_string());

    let target_folder = PathBuf::from(base_dir)
        .join(artist_folder_name)
        .join(album_folder_name);

    let destination_path = target_folder.join(file_name);

    std::fs::create_dir_all(&target_folder).unwrap();
    std::fs::rename(file_path, &destination_path)
        .map(|_| {
            println!("File moved successfully");
            track.file_path = Some(destination_path);
            ()
        })
        .map_err(|e| Error::Custom(format!("Failed to move file: {}", e)))
}