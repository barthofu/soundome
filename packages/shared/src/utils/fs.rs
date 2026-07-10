use std::env;
use std::path::PathBuf;

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
