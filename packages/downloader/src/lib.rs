pub mod youtube;

use std::path::PathBuf;
use async_trait::async_trait;
use shared::models::track::Track;
use shared::errors::Error;

// this is the trait that all downloaders must implement
#[async_trait]
pub trait Provider {

    async fn search(&self, track: Track) -> Option<String>;
    async fn download(&mut self, url: &str) -> Result<PathBuf, Error>;
}
