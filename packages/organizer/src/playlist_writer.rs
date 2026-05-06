use std::{
    io::Write,
    path::{Path, PathBuf},
};

use shared::{
    errors::Error,
    models::{Playlist, Track},
    types::SoundomeResult,
};

/// Sanitize a playlist name into a safe filesystem file name by replacing
/// characters that are problematic on common operating systems.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

/// Write an M3U8 playlist file to `output_dir`.
///
/// - Creates `output_dir` when it does not exist.
/// - Derives the file name from `playlist.name` (sanitized).
/// - Overwrites any existing file with the same name.
/// - Skips tracks that have no `file_path` (not yet finalized) or that still
///   require manual validation (`needs_validation == true`).
///
/// Returns the path of the written file.
pub fn write_m3u8(playlist: &Playlist, tracks: &[Track], output_dir: &Path) -> SoundomeResult<PathBuf> {
    std::fs::create_dir_all(output_dir).map_err(|e| {
        Error::Custom(format!(
            "Failed to create M3U8 output directory {:?}: {}",
            output_dir, e
        ))
    })?;

    let file_name = format!("{}.m3u8", sanitize_name(&playlist.name));
    let file_path = output_dir.join(&file_name);

    let mut file = std::fs::File::create(&file_path).map_err(|e| {
        Error::Custom(format!("Failed to create M3U8 file {:?}: {}", file_path, e))
    })?;

    writeln!(file, "#EXTM3U").map_err(|e| Error::Custom(format!("M3U8 write error: {}", e)))?;
    writeln!(file, "#EXTENC:UTF-8").map_err(|e| Error::Custom(format!("M3U8 write error: {}", e)))?;

    for track in tracks {
        // Skip tracks that have not been finalized or require validation.
        if track.needs_validation {
            continue;
        }
        let Some(ref path) = track.file_path else {
            continue;
        };

        // Duration in seconds; -1 when unknown (valid per M3U8 spec).
        let duration_secs = track
            .duration
            .map(|ms| ms / 1000)
            .unwrap_or(-1);

        // Display string: "Artist1, Artist2 - Title"
        let artist_str = if track.artists.is_empty() {
            "Unknown Artist".to_string()
        } else {
            track
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let display = format!("{} - {}", artist_str, track.title);

        writeln!(file, "#EXTINF:{},{}", duration_secs, display)
            .map_err(|e| Error::Custom(format!("M3U8 write error: {}", e)))?;
        writeln!(file, "{}", path.display())
            .map_err(|e| Error::Custom(format!("M3U8 write error: {}", e)))?;
    }

    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use shared::models::{Artist, Playlist, Track};

    use super::*;

    fn make_playlist(name: &str) -> Playlist {
        Playlist {
            id: Some(1),
            name: name.to_string(),
            source: shared::models::Platform::Spotify,
            source_url: None,
            cover: None,
        }
    }

    fn make_track(title: &str, artist: &str, duration_ms: Option<i32>, path: Option<&str>) -> Track {
        Track {
            id: Some(1),
            title: title.to_string(),
            artists: vec![Artist {
                id: None,
                name: artist.to_string(),
                icon: None,
                references: vec![],
            }],
            album: None,
            date: None,
            genre: None,
            cover: None,
            duration: duration_ms,
            track_number: None,
            disc_number: None,
            label: None,
            file_path: path.map(PathBuf::from),
            references: vec![],
            needs_validation: false,
            validation_reason: None,
        }
    }

    #[test]
    fn test_write_m3u8_basic() {
        let dir = tempfile::tempdir().unwrap();
        let playlist = make_playlist("My Playlist");
        let tracks = vec![
            make_track("Windowlicker", "Aphex Twin", Some(213_000), Some("/library/Aphex Twin/Windowlicker.flac")),
            make_track("Archangel", "Burial", Some(187_000), Some("/library/Burial/Archangel.flac")),
        ];

        let path = write_m3u8(&playlist, &tracks, dir.path()).unwrap();
        assert_eq!(path.file_name().unwrap(), "My Playlist.m3u8");

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("#EXTM3U\n"));
        assert!(content.contains("#EXTENC:UTF-8\n"));
        assert!(content.contains("#EXTINF:213,Aphex Twin - Windowlicker\n"));
        assert!(content.contains("/library/Aphex Twin/Windowlicker.flac\n"));
        assert!(content.contains("#EXTINF:187,Burial - Archangel\n"));
    }

    #[test]
    fn test_write_m3u8_skips_unfinalized_and_needs_validation() {
        let dir = tempfile::tempdir().unwrap();
        let playlist = make_playlist("Test");

        let mut needs_val = make_track("Draft", "Artist", None, Some("/library/draft.flac"));
        needs_val.needs_validation = true;

        let no_path = make_track("No Path", "Artist", None, None);

        let good = make_track("Good", "Artist", Some(60_000), Some("/library/good.flac"));

        let path = write_m3u8(&playlist, &[needs_val, no_path, good], dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains("Draft"));
        assert!(!content.contains("No Path"));
        assert!(content.contains("Good"));
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("My/Playlist:Name"), "My_Playlist_Name");
        assert_eq!(sanitize_name("Normal Name"), "Normal Name");
    }

    #[test]
    fn test_write_m3u8_unknown_duration() {
        let dir = tempfile::tempdir().unwrap();
        let playlist = make_playlist("NoDuration");
        let tracks = vec![make_track("Song", "Artist", None, Some("/lib/song.flac"))];

        let path = write_m3u8(&playlist, &tracks, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("#EXTINF:-1,Artist - Song\n"));
    }
}
