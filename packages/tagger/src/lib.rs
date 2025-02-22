use shared::{models::track::Track, utils::enums::Match};

pub mod providers;
pub mod file;

pub trait TagProvider {
    fn get(&self, track: &Track) -> impl std::future::Future<Output = Match<Track>> + Send;
}
