use std::sync::Arc;

use diesel::SqliteConnection;
use shared::types::SoundomeResult;

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

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<shared::models::Track> {
        self.track_repo.get_by_id(conn, id)
    }

    pub fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<shared::models::Track>> {
        self.track_repo.get_all(conn)
    }

    pub fn create(&self, conn: &mut SqliteConnection, new_track: &shared::models::Track) -> SoundomeResult<shared::models::Track> {
        self.track_repo.create(conn, new_track)
    }

    pub fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &shared::models::Track) -> SoundomeResult<shared::models::Track> {
        self.track_repo.update(conn, id, updated_track)
    }

    // Getters

    pub fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<shared::models::Track> {
        self.track_repo.get_by_url(conn, url).ok()
    }

    // Custom

    /// Finds a track by comparing title and artists using a similarity metric.
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

    /// Creates a track along with its associated artists, album, and references.
    pub fn create_or_ignore(&self, conn: &mut SqliteConnection, track: &shared::models::Track) -> SoundomeResult<shared::models::Track> {
        // Step 1: Handle album - use create_or_ignore
        let album_id = if let Some(album) = &track.album {
            let created_album = self.album_repo.create_or_ignore(conn, album)?;
            let album_id = created_album.id.unwrap();
            
            // Handle album artists using create_or_ignore
            for artist in &album.artists {
                let created_artist = self.artist_repo.create_or_ignore(conn, artist)?;
                let artist_id = created_artist.id.unwrap();
                
                // Create artist-album relationship
                self.artist_repo.create_album_relationship(conn, artist_id, album_id)?;
            }
            
            Some(album_id)
        } else {
            None
        };
        
        // Step 2: Create the track using repository
        // First, create a track copy with the resolved album_id
        let mut track_to_create = track.clone();
        if let Some(album_id) = album_id {
            // If we have an album_id, create an album instance with just the ID
            track_to_create.album = Some(shared::models::Album {
                id: Some(album_id),
                title: String::new(), // Repository will ignore these when converting
                artists: Vec::new(),
                album_type: shared::models::AlbumType::Album,
                cover: None,
                date: None,
                references: Vec::new(),
            });
        }
        
        let created_track = self.track_repo.create(conn, &track_to_create)?;
        let track_id = created_track.id.unwrap();
        
        // Step 3: Handle track artists using create_or_ignore
        for artist in &track.artists {
            let created_artist = self.artist_repo.create_or_ignore(conn, artist)?;
            let artist_id = created_artist.id.unwrap();
            
            // Create artist-track relationship
            self.artist_repo.create_track_relationship(conn, artist_id, track_id)?;
        }
        
        // Step 4: Create track references
        self.track_repo.create_references(conn, track_id, &track.references)?;
        
        // Step 5: Load the complete track with all relationships for return
        self.track_repo.get_by_id(conn, track_id)
    }
}
