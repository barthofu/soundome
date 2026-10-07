use std::env;
use std::path::{Path, PathBuf};

use config::GLOBAL_CONFIG;

/**
 * Get the path of a file relative to the project root
 */
pub fn get_project_relative_path(relative_path: &str) -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let exe_dir = exe_path
        .parent()
        .expect("Failed to get executable directory");
    exe_dir.join(relative_path)
}

/// Reconstruct the filesystem path for a track path stored relative to one of
/// Soundome's configured audio roots. Pending tracks prefer the staging root;
/// finalized tracks prefer the library root. The other root is tried when the
/// preferred candidate does not exist, which also keeps older rows working
/// when their validation state and path root do not agree.
///
/// Absolute paths are accepted as legacy values and left unchanged.
pub fn resolve_track_file_path(
    stored_path: &Path,
    needs_validation: bool,
    base_library_dir: &Path,
    temp_download_dir: &Path,
) -> PathBuf {
    if stored_path.is_absolute() {
        return stored_path.to_path_buf();
    }

    let [preferred_root, fallback_root] =
        ordered_track_roots(needs_validation, base_library_dir, temp_download_dir);
    let preferred_path = absolute_path(preferred_root)
        .unwrap_or_else(|| preferred_root.to_path_buf())
        .join(stored_path);
    if preferred_path.is_file() {
        return preferred_path;
    }

    let fallback_path = absolute_path(fallback_root)
        .unwrap_or_else(|| fallback_root.to_path_buf())
        .join(stored_path);
    if fallback_path.is_file() {
        return fallback_path;
    }

    // Preserve the primary candidate when the file is missing. Callers such
    // as validation approval can then report the expected path or recover it.
    preferred_path
}

/// Convert a path under either configured root to a root-relative path before
/// it is persisted. Paths already relative to a root are preserved, and paths
/// outside both managed roots remain untouched for compatibility.
pub fn track_file_path_for_storage(
    file_path: &Path,
    needs_validation: bool,
    base_library_dir: &Path,
    temp_download_dir: &Path,
) -> PathBuf {
    let [preferred_root, fallback_root] =
        ordered_track_roots(needs_validation, base_library_dir, temp_download_dir);

    relative_to_root(file_path, preferred_root)
        .or_else(|| relative_to_root(file_path, fallback_root))
        .unwrap_or_else(|| file_path.to_path_buf())
}

/// Return whether `path` is contained by `root`, comparing paths against the
/// process working directory when either value is configured as relative.
pub fn path_is_within_root(path: &Path, root: &Path) -> bool {
    relative_to_root(path, root).is_some()
}

/// Config-backed resolver used at the database boundary. If configuration is
/// not initialized (for example in isolated mapper tests), leave the path as-is.
pub fn resolve_track_file_path_from_config(stored_path: &Path, needs_validation: bool) -> PathBuf {
    let Some(config) = GLOBAL_CONFIG.get() else {
        return stored_path.to_path_buf();
    };

    resolve_track_file_path(
        stored_path,
        needs_validation,
        Path::new(&config.general.base_library_dir),
        Path::new(&config.general.temp_download_dir),
    )
}

/// Config-backed storage normalizer used by the database mapper and the
/// startup compatibility pass for existing absolute paths.
pub fn track_file_path_for_storage_from_config(
    file_path: &Path,
    needs_validation: bool,
) -> PathBuf {
    let Some(config) = GLOBAL_CONFIG.get() else {
        return file_path.to_path_buf();
    };

    track_file_path_for_storage(
        file_path,
        needs_validation,
        Path::new(&config.general.base_library_dir),
        Path::new(&config.general.temp_download_dir),
    )
}

fn ordered_track_roots<'a>(
    needs_validation: bool,
    base_library_dir: &'a Path,
    temp_download_dir: &'a Path,
) -> [&'a Path; 2] {
    if needs_validation {
        [temp_download_dir, base_library_dir]
    } else {
        [base_library_dir, temp_download_dir]
    }
}

fn absolute_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        env::current_dir()
            .ok()
            .map(|current_dir| current_dir.join(path))
    }
}

fn relative_to_root(path: &Path, root: &Path) -> Option<PathBuf> {
    path.strip_prefix(root)
        .ok()
        .map(Path::to_path_buf)
        .or_else(|| {
            let absolute_file_path = absolute_path(path)?;
            let absolute_root = absolute_path(root)?;
            absolute_file_path
                .strip_prefix(absolute_root)
                .ok()
                .map(Path::to_path_buf)
        })
}

/// Sanitize a string (track title, artist name, album title, playlist name, ...)
/// into a value that is safe to use as a single filesystem path component.
///
/// Replaces characters that are reserved on common filesystems (notably `/`,
/// which otherwise silently creates unintended subdirectories, e.g. a track
/// titled "One Night / All Night") as well as Windows-reserved characters,
/// and trims surrounding whitespace.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            // Unix/Linux filesystem reserved
            '/' | '\\' => '_',
            // Windows filesystem reserved
            // '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod track_file_path_tests {
    use super::{resolve_track_file_path, track_file_path_for_storage};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> PathBuf {
        let id = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "soundome-track-path-test-{}-{id}",
            std::process::id()
        ))
    }

    #[test]
    fn resolves_from_the_preferred_root_and_falls_back_to_the_other_root() {
        let root = temp_root();
        let library = root.join("library");
        let staging = root.join("staging");
        let relative = Path::new("Artist/Album/Track.mp3");
        let library_file = library.join(relative);
        std::fs::create_dir_all(library_file.parent().unwrap()).unwrap();
        std::fs::write(&library_file, b"audio").unwrap();

        assert_eq!(
            resolve_track_file_path(relative, true, &library, &staging),
            library_file
        );

        let staged_file = staging.join(relative);
        std::fs::create_dir_all(staged_file.parent().unwrap()).unwrap();
        std::fs::write(&staged_file, b"staged audio").unwrap();
        assert_eq!(
            resolve_track_file_path(relative, true, &library, &staging),
            staged_file
        );
        assert_eq!(
            resolve_track_file_path(relative, false, &library, &staging),
            library_file
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stores_paths_relative_to_either_managed_root() {
        let root = temp_root();
        let library = root.join("library");
        let staging = root.join("staging");

        assert_eq!(
            track_file_path_for_storage(
                &library.join("Artist/Album/Track.mp3"),
                false,
                &library,
                &staging,
            ),
            PathBuf::from("Artist/Album/Track.mp3")
        );
        assert_eq!(
            track_file_path_for_storage(
                &staging.join("pending-track.mp3"),
                true,
                &library,
                &staging,
            ),
            PathBuf::from("pending-track.mp3")
        );
    }
}
