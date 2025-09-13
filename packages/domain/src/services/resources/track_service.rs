use std::sync::Arc;

use diesel::SqliteConnection;

use crate::ports::repositories::{AlbumRepository, ArtistRepository, TrackRepository};

pub struct TrackService {
    track_repo: Arc<dyn TrackRepository + Send + Sync>,
    album_repo: Arc<dyn AlbumRepository + Send + Sync>,
    artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
}

impl TrackService {

    const SIMILARITY_THRESHOLD: f64 = 0.8;

    pub fn new(
        track_repo: Arc<dyn TrackRepository + Send + Sync>,
        album_repo: Arc<dyn AlbumRepository + Send + Sync>,
        artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
    ) -> Self {
        Self {
            track_repo,
            album_repo,
            artist_repo,
        }
    }

    // CRUD

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> shared::types::SoundomeResult<shared::models::Track> {
        self.track_repo.get_by_id(conn, id)
    }

    pub fn get_all(&self, conn: &mut SqliteConnection) -> shared::types::SoundomeResult<Vec<shared::models::Track>> {
        self.track_repo.get_all(conn)
    }

    pub fn create(&self, conn: &mut SqliteConnection, new_track: &shared::models::Track) -> shared::types::SoundomeResult<shared::models::Track> {
        self.track_repo.create(conn, new_track)
    }

    pub fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &shared::models::Track) -> shared::types::SoundomeResult<shared::models::Track> {
        self.track_repo.update(conn, id, updated_track)
    }

    // Getters

    pub fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<shared::models::Track> {
        self.track_repo.get_by_url(conn, url).ok()
    }

    // Custom

    pub fn find_track_by_title_and_artist(
        &self,
        conn: &mut SqliteConnection,
        track: &shared::models::Track,
    ) -> Option<shared::models::Track> {

        // First, we need to get comparative tracks
        let comparative_tracks = self.get_all(conn).ok()?;
        let mut best_match: Option<(&shared::models::Track, f64)> = None;

        // Then, we iterate through them to find the best match using the .compare method
        for comparative_track in &comparative_tracks {
            let score = track.compare(comparative_track);
            if let Some((_, best_score)) = &best_match {
                if score > *best_score {
                    best_match = Some((comparative_track, score));
                }
            } else {
                best_match = Some((comparative_track, score));
            }
        }

        // Finally, we return the best match if its score is above a certain threshold
        best_match
            .filter(|&(_, score)| score >= Self::SIMILARITY_THRESHOLD)
            .map(|(track, _)| track.clone())
    }
}
