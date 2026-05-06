use std::path::PathBuf;

use shared::{errors::Error, models::Track, types::SoundomeResult};
use std::fs;

/// Moves the track file to the organized library structure based on artist and album.
/// Updates the track's file_path to the new location.
/// If the destination file already exists, it will be replaced.
pub fn move_track_file(track: &mut Track, base_library_dir: &str) -> SoundomeResult<()> {
    tracing::info!("Moving track file: {:?}", track.file_path);

    let file_path = track
        .file_path
        .as_ref()
        .ok_or(Error::Custom("Track file path is missing".to_string()))?;
    let file_name = file_path
        .file_name()
        .ok_or(Error::Custom("Invalid file path".to_string()))?;

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

    let target_folder = PathBuf::from(base_library_dir)
        .join(artist_folder_name)
        .join(album_folder_name);

    let destination_path = target_folder.join(file_name);

    fs::create_dir_all(&target_folder).unwrap();

    // If destination exists, remove it first to force replace
    if destination_path.exists() {
        fs::remove_file(&destination_path)
            .map_err(|e| Error::Custom(format!("Failed to remove existing file: {}", e)))?;
    }

    fs::rename(file_path, &destination_path)
        .map(|_| {
            tracing::info!("File moved successfully");
            track.file_path = Some(destination_path);
        })
        .map_err(|e| Error::Custom(format!("Failed to move file: {}", e)))
}
