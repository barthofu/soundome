use std::sync::Arc;

use diesel::SqliteConnection;
use shared::{models::{Album, AlbumType, Track}, types::SoundomeResult};

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

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Track> {
        self.track_repo.get_by_id(conn, id)
    }

    pub fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>> {
        self.track_repo.get_all(conn)
    }

    pub fn create(&self, conn: &mut SqliteConnection, new_track: &Track) -> SoundomeResult<Track> {
        self.track_repo.create(conn, new_track)
    }

    pub fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &Track) -> SoundomeResult<Track> {
        self.track_repo.update(conn, id, updated_track)
    }

    // Getters

    pub fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<Track> {
        self.track_repo.get_by_url(conn, url).ok()
    }

    // Custom

    /// Finds a track by comparing title and artists using a similarity metric.
    pub fn find_track_by_title_and_artist(
        &self,
        conn: &mut SqliteConnection,
        track: &Track,
    ) -> Option<Track> {

        // First, we need to get comparative tracks
        let comparative_tracks = self.get_all(conn).ok()?;
        let mut best_match: Option<(&Track, f64)> = None;

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

    /// Creates or updates a track in the database along with its associated artists, album, and references.
    /// Si une entité existe (par ID ou clé unique), elle est mise à jour, sinon créée. Les relations sont maintenues.
    pub fn create_or_update(&self, conn: &mut SqliteConnection, track: &Track) -> SoundomeResult<Track> {
        // Step 1: Album (create or update)
        let album_id = if let Some(album) = &track.album {
            let album_to_save = album.clone();
            let saved_album = if let Some(id) = album.id {
                // update album
                self.album_repo.update(conn, id, &album_to_save)?
            } else {
                // try to find by unique fields (e.g. title, artists, date)
                match self.album_repo.find_by_unique_fields(conn, &album_to_save) {
                    Ok(Some(existing_album)) => {
                        self.album_repo.update(conn, existing_album.id.unwrap(), &album_to_save)?
                    },
                    _ => self.album_repo.create(conn, &album_to_save)?
                }
            };
            let album_id = saved_album.id.unwrap();
            // Album artists (create or update)
            for artist in &album.artists {
                let saved_artist = if let Some(id) = artist.id {
                    self.artist_repo.update(conn, id, artist)?
                } else {
                    match self.artist_repo.find_by_unique_fields(conn, artist) {
                        Ok(Some(existing_artist)) => self.artist_repo.update(conn, existing_artist.id.unwrap(), artist)?,
                        _ => self.artist_repo.create(conn, artist)?
                    }
                };
                let artist_id = saved_artist.id.unwrap();
                self.artist_repo.create_album_relationship(conn, artist_id, album_id)?;
            }
            Some(album_id)
        } else {
            None
        };

        // Step 2: Track (create or update)
        let mut track_to_save = track.clone();
        if let Some(album_id) = album_id {
            track_to_save.album = Some(Album {
                id: Some(album_id),
                title: String::new(),
                artists: Vec::new(),
                album_type: AlbumType::Album,
                cover: None,
                date: None,
                references: Vec::new(),
            });
        }
        let saved_track = if let Some(id) = track.id {
            self.track_repo.update(conn, id, &track_to_save)?
        } else {
            match self.track_repo.find_by_unique_fields(conn, &track_to_save) {
                Ok(Some(existing_track)) => self.track_repo.update(conn, existing_track.id.unwrap(), &track_to_save)?,
                _ => self.track_repo.create(conn, &track_to_save)?
            }
        };
        let track_id = saved_track.id.unwrap();

        // Step 3: Track artists (create or update)
        for artist in &track.artists {
            let saved_artist = if let Some(id) = artist.id {
                self.artist_repo.update(conn, id, artist)?
            } else {
                match self.artist_repo.find_by_unique_fields(conn, artist) {
                    Ok(Some(existing_artist)) => self.artist_repo.update(conn, existing_artist.id.unwrap(), artist)?,
                    _ => self.artist_repo.create(conn, artist)?
                }
            };
            let artist_id = saved_artist.id.unwrap();
            self.artist_repo.create_track_relationship(conn, artist_id, track_id)?;
        }

        // Step 4: Track references (replace all)
        self.track_repo.create_references(conn, track_id, &track.references)?;

        // Step 5: Reload full track
        self.track_repo.get_by_id(conn, track_id)
    }

    /// Compares file quality of two tracks.
    /// Currently, this is a simple comparison based on bitrate.
    /// 
    /// Returns true if the new track has better quality.
    pub fn is_better_quality(&self, existing_track: &Track, new_track: &Track) -> bool {

        let existing_bitrate = existing_track.get_bitrate();
        let new_bitrate = new_track.get_bitrate();

        match (existing_bitrate, new_bitrate) {
            (Some(e), Some(n)) => n > e,
            // if we can't determine, default to false
            _ => false,
        }
    }
    
}
