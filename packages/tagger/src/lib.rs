use shared::{models::Track, utils::enums::Match};

pub mod enricher;
pub mod file;
pub mod providers;

pub trait TagProvider {
    fn get_best_match_from_track(
        &self,
        track: &Track,
    ) -> impl std::future::Future<Output = Match<Track>> + Send;
    fn get_matches_from_query(
        &self,
        query: &str,
    ) -> impl std::future::Future<Output = Vec<Track>> + Send;
    fn get_match_from_query(
        &self,
        query: &str,
    ) -> impl std::future::Future<Output = Match<Track>> + Send;
}
