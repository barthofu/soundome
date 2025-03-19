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
