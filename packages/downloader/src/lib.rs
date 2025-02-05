pub mod youtube;

use std::path::PathBuf;
use async_trait::async_trait;
use crate::models::track::Track;
use super::errors::Error;

#[async_trait]
pub trait Provider {

    async fn search(&self, track: Track) -> Option<String>;
    async fn download(&mut self, url: &str) -> Result<PathBuf, Error>;
}
