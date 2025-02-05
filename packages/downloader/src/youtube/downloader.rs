use std::path::PathBuf;

use async_trait::async_trait;

use super::Youtube;

#[async_trait]
pub trait Downloader {
    fn get_downloader() -> PathBuf;
}

#[async_trait]
impl Downloader for Youtube {

    fn get_downloader() -> PathBuf {

        PathBuf::from("core/drivers/yt-dlp")
    }
}
