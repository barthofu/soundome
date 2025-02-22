use std::path::PathBuf;
use shared::{errors::Error, models::track::Track};

pub mod providers;

pub trait TagProvider {
    fn search(&self, track: &Track) -> impl std::future::Future<Output = Vec<Track>> + Send;
}

pub trait TagWriter {
    fn write(&self, file_path: &PathBuf, track: &Track) -> Result<(), Error>;
}
