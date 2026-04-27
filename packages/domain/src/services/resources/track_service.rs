use std::sync::Arc;

use diesel::{Connection, SqliteConnection};
use shared::{errors::Error, models::{Album, AlbumType, ReferenceType, Track}, types::SoundomeResult};

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

    pub fn delete_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        self.track_repo.delete(conn, id)
    }

    // Getters

    pub fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<Track> {
        self.track_repo.get_by_url(conn, url).ok()
    }

    pub fn get_pending_validations(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Track>> {
        self.track_repo.get_pending_validations(conn)
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
        conn.transaction(|tx| {
            // 1) Album (+ artistes + références album)
            let album_id_opt = if let Some(album) = &track.album {
                // create or update album
                let saved_album = if let Some(id) = album.id {
                    self.album_repo.update(tx, id, album)?
                } else {
                    self.album_repo.create(tx, album)?
                };
                let album_id = saved_album.id.ok_or_else(|| Error::Internal("missing album id after create/update".into()))?;

                // Upsert artists of album and collect IDs
                let mut album_artist_ids: Vec<i32> = Vec::with_capacity(album.artists.len());
                for artist in &album.artists {
                    let saved_artist = if let Some(id) = artist.id {
                        self.artist_repo.update(tx, id, artist)?
                    } else {
                        self.artist_repo.create(tx, artist)?
                    };
                    let artist_id = saved_artist.id.ok_or_else(|| Error::Internal("missing artist id after create/update".into()))?;
                    // album/artist refs are stored as metadata only
                    let mut refs = artist.references.clone();
                    for r in &mut refs {
                        r.ref_type = ReferenceType::Metadata;
                        r.id = None;
                    }
                    self.artist_repo.set_references(tx, artist_id, &refs)?;
                    album_artist_ids.push(artist_id);
                }
                // Replace album artists relationships only if the caller provided them
                if !album.artists.is_empty() {
                    self.artist_repo.set_album_artists(tx, album_id, &album_artist_ids)?;
                }
                // Replace/merge album references (metadata only)
                let mut album_refs = album.references.clone();
                for r in &mut album_refs {
                    r.ref_type = ReferenceType::Metadata;
                    r.id = None;
                }
                self.album_repo.set_references(tx, album_id, &album_refs)?;

                Some(album_id)
            } else {
                None
            };

            // 2) Track (lier à l'album si présent)
            let mut track_to_save = track.clone();
            if let Some(album_id) = album_id_opt {
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
                self.track_repo.update(tx, id, &track_to_save)?
            } else {
                self.track_repo.create(tx, &track_to_save)?
            };
            let track_id = saved_track.id.ok_or_else(|| Error::Internal("missing track id after create/update".into()))?;

            // 3) Artistes du track (remplacement)
            let mut track_artist_ids: Vec<i32> = Vec::with_capacity(track.artists.len());
            for artist in &track.artists {
                let saved_artist = if let Some(id) = artist.id {
                    self.artist_repo.update(tx, id, artist)?
                } else {
                    self.artist_repo.create(tx, artist)?
                };
                let artist_id = saved_artist.id.ok_or_else(|| Error::Internal("missing artist id after create/update".into()))?;
                // artist refs are stored as metadata only
                let mut refs = artist.references.clone();
                for r in &mut refs {
                    r.ref_type = ReferenceType::Metadata;
                    r.id = None;
                }
                self.artist_repo.set_references(tx, artist_id, &refs)?;
                track_artist_ids.push(artist_id);
            }
            self.artist_repo.set_track_artists(tx, track_id, &track_artist_ids)?;

            // 4) Références du track (remplacement)
            self.track_repo.set_references(tx, track_id, &track.references)?;

            // 5) Reload complet
            self.track_repo.get_by_id(tx, track_id)
        })
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

    /// Delete track file
    pub fn delete_track_file(&self, track: &Track) -> SoundomeResult<bool> {
        let file_deleted = if let Some(file_path) = &track.file_path {
            std::fs::remove_file(file_path).is_ok()
        } else {
            false
        };
        
        Ok(file_deleted)
    }
    
}
