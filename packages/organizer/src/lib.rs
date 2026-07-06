use std::path::PathBuf;

use shared::{errors::Error, models::Track, types::SoundomeResult};
use std::fs;

pub mod playlist_writer;

/// Attempts to remove empty directories up the tree until a non-empty directory or base is reached.
/// This is used for cleanup after moving a track file (e.g., removing empty artist/album folders).
pub fn cleanup_empty_parent_dirs(file_path: &PathBuf, base_library_dir: &str) -> SoundomeResult<()> {
    let base_path = PathBuf::from(base_library_dir);
    let mut current = file_path.parent().map(|p| p.to_path_buf());

    while let Some(parent_dir) = current {
        // Stop if we've exited the base library directory (safety check)
        if !parent_dir.starts_with(&base_path) || parent_dir == base_path {
            break;
        }

        // Try to remove the directory if it's empty
        match fs::remove_dir(&parent_dir) {
            Ok(_) => {
                tracing::debug!("Removed empty directory: {:?}", parent_dir);
                current = parent_dir.parent().map(|p| p.to_path_buf());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Directory already gone, continue up
                current = parent_dir.parent().map(|p| p.to_path_buf());
            }
            Err(e) if e.kind() == std::io::ErrorKind::Other => {
                // ENOTEMPTY or other errors — directory not empty, stop here
                tracing::debug!("Directory not empty or error, stopping cleanup: {:?}", parent_dir);
                break;
            }
            Err(e) => {
                // Log other errors but don't fail — cleanup is best-effort
                tracing::warn!("Could not remove directory {:?}: {}", parent_dir, e);
                break;
            }
        }
    }

    Ok(())
}

/// Moves the track file to the organized library structure based on artist and album.
/// Updates the track's file_path to the new location.
/// If the destination file already exists, it will be replaced.
pub fn move_track_file(track: &mut Track, base_library_dir: &str) -> SoundomeResult<()> {
    tracing::info!("Moving track file: {:?}", track.file_path);

    let file_path = track
        .file_path
        .as_ref()
        .ok_or(Error::Custom("Track file path is missing".to_string()))?;

    // Extract the file extension from the old file path
    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("mp3");

    // Build the new filename using the current track title and the original extension
    let new_file_name = format!("{}.{}", track.title, file_extension);

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

    let destination_path = target_folder.join(new_file_name);

    fs::create_dir_all(&target_folder).unwrap();

    // If destination exists, remove it first to force replace
    if destination_path.exists() {
        fs::remove_file(&destination_path)
            .map_err(|e| Error::Custom(format!("Failed to remove existing file: {}", e)))?;
    }

    let old_path = file_path.clone();
    fs::rename(file_path, &destination_path)
        .map(|_| {
            tracing::info!("File moved successfully");
            track.file_path = Some(destination_path);
        })
        .map_err(|e| Error::Custom(format!("Failed to move file: {}", e)))?;

    // Best-effort cleanup: remove empty parent directories from the old location
    if let Err(e) = cleanup_empty_parent_dirs(&old_path, base_library_dir) {
        tracing::warn!("Cleanup of empty directories failed: {}", e);
    }

    Ok(())
}
