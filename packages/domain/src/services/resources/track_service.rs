use std::sync::Arc;

use diesel::{Connection, SqliteConnection};
use shared::{
    errors::Error,
    models::{Album, AlbumType, Artist, Reference, ReferenceType, Track},
    types::SoundomeResult,
};

use crate::ports::repositories::{AlbumRepository, ArtistRepository, TrackRepository};
use crate::services::resources::track_ops::delete_track_with_cascade;

/// Patch applied when a user approves a pending validation.
/// All fields are optional; only provided fields overwrite the existing value.
pub struct ValidationPatch {
    pub title: Option<String>,
    /// Replaces the track's artists when provided (list of names).
    pub artists: Option<Vec<String>>,
    /// Updates or creates the album title when provided.
    pub album_title: Option<String>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    /// YouTube or YouTube Music URL to download from.
    /// Required when the track has no staged file (e.g. SoundCloud DRM protection).
    pub provider_url: Option<String>,
}

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

    pub fn update(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        updated_track: &Track,
    ) -> SoundomeResult<Track> {
        self.track_repo.update(conn, id, updated_track)
    }

    /// Delete a track by ID.
    ///
    /// After removing the track row, checks whether its album and each of its
    /// artists have become orphans (no remaining tracks).  Orphaned albums and
    /// artists are deleted automatically inside the same transaction.
    pub fn delete_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        delete_track_with_cascade(
            conn,
            id,
            &self.track_repo,
            &self.album_repo,
            &self.artist_repo,
        )
    }

    // Getters

    pub fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> Option<Track> {
        self.track_repo.get_by_url(conn, url).ok()
    }

    pub fn get_recent(
        &self,
        conn: &mut SqliteConnection,
        limit: i64,
    ) -> SoundomeResult<Vec<Track>> {
        self.track_repo.get_recent(conn, limit)
    }

    pub fn get_pending_validations(
        &self,
        conn: &mut SqliteConnection,
    ) -> SoundomeResult<Vec<Track>> {
        self.track_repo.get_pending_validations(conn)
    }

    pub fn count(&self, conn: &mut SqliteConnection) -> SoundomeResult<i64> {
        self.track_repo.count(conn)
    }

    pub fn count_pending_validations(&self, conn: &mut SqliteConnection) -> SoundomeResult<i64> {
        self.track_repo.count_pending_validations(conn)
    }

    /// Applies `patch` to an existing track, clears its validation flag, and persists.
    pub fn validate_track(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        patch: ValidationPatch,
    ) -> SoundomeResult<Track> {
        conn.transaction(|tx| {
            let mut track = self.track_repo.get_by_id(tx, id)?;

            if let Some(title) = patch.title {
                track.title = title;
            }
            if let Some(genre) = patch.genre {
                track.genre = Some(genre);
            }
            if let Some(date) = patch.date {
                track.date = Some(date);
            }
            if let Some(tn) = patch.track_number {
                track.track_number = Some(tn);
            }
            if let Some(dn) = patch.disc_number {
                track.disc_number = Some(dn);
            }
            if let Some(label) = patch.label {
                track.label = Some(label);
            }

            if let Some(names) = patch.artists {
                let mut artists: Vec<Artist> = Vec::with_capacity(names.len());
                for name in names {
                    let artist = Artist {
                        id: None,
                        name,
                        icon: None,
                        references: vec![],
                    };
                    let saved = self.artist_repo.create_or_ignore(tx, &artist)?;
                    artists.push(saved);
                }
                track.artists = artists;
            }

            if let Some(album_title) = patch.album_title {
                match track.album.as_mut() {
                    Some(album) => album.title = album_title,
                    None => {
                        track.album = Some(Album {
                            id: None,
                            title: album_title,
                            artists: vec![],
                            album_type: AlbumType::Album,
                            cover: None,
                            date: None,
                            references: vec![],
                        });
                    }
                }
            }

            track.needs_validation = false;
            track.validation_reason = None;

            self.create_or_update(tx, &track)
        })
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
    pub fn create_or_update(
        &self,
        conn: &mut SqliteConnection,
        track: &Track,
    ) -> SoundomeResult<Track> {
        conn.transaction(|tx| {
            // 1) Album (+ artistes + références album)
            let album_id_opt = if let Some(album) = &track.album {
                // create or update album
                let saved_album = if let Some(id) = album.id {
                    self.album_repo.update(tx, id, album)?
                } else {
                    // Deduplicate by (title, artist names): two albums share a row only
                    // when both their title AND at least one artist name match.
                    // This prevents "Greatest Hits" from two unrelated artists
                    // from collapsing into the same DB row.
                    let artist_names: Vec<String> =
                        album.artists.iter().map(|a| a.name.clone()).collect();
                    match self.album_repo.find_by_title_and_artists(
                        tx,
                        &album.title,
                        &artist_names,
                    )? {
                        Some(existing) => existing,
                        None => {
                            let created = self.album_repo.create(tx, album)?;
                            let aid = created.id.ok_or_else(|| {
                                Error::Internal("missing album id after create".into())
                            })?;
                            self.album_repo
                                .create_references(tx, aid, &album.references)?;
                            self.album_repo.get_by_id(tx, aid)?
                        }
                    }
                };
                let album_id = saved_album.id.ok_or_else(|| {
                    Error::Internal("missing album id after create/update".into())
                })?;

                // Upsert artists of album and collect IDs
                let mut album_artist_ids: Vec<i32> = Vec::with_capacity(album.artists.len());
                for artist in &album.artists {
                    let saved_artist = if let Some(id) = artist.id {
                        self.artist_repo.update(tx, id, artist)?
                    } else {
                        self.artist_repo.create_or_ignore(tx, artist)?
                    };
                    let artist_id = saved_artist.id.ok_or_else(|| {
                        Error::Internal("missing artist id after create/update".into())
                    })?;
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
                    self.artist_repo
                        .set_album_artists(tx, album_id, &album_artist_ids)?;
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
            let track_id = saved_track
                .id
                .ok_or_else(|| Error::Internal("missing track id after create/update".into()))?;

            // 3) Artistes du track (remplacement)
            let mut track_artist_ids: Vec<i32> = Vec::with_capacity(track.artists.len());
            for artist in &track.artists {
                let saved_artist = if let Some(id) = artist.id {
                    self.artist_repo.update(tx, id, artist)?
                } else {
                    self.artist_repo.create_or_ignore(tx, artist)?
                };
                let artist_id = saved_artist.id.ok_or_else(|| {
                    Error::Internal("missing artist id after create/update".into())
                })?;
                // artist refs are stored as metadata only
                let mut refs = artist.references.clone();
                for r in &mut refs {
                    r.ref_type = ReferenceType::Metadata;
                    r.id = None;
                }
                self.artist_repo.set_references(tx, artist_id, &refs)?;
                track_artist_ids.push(artist_id);
            }
            self.artist_repo
                .set_track_artists(tx, track_id, &track_artist_ids)?;

            // 4) Références du track (remplacement)
            self.track_repo
                .set_references(tx, track_id, &track.references)?;

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

    /// Append a single reference to a track and return the full updated list.
    pub fn add_reference(
        &self,
        conn: &mut SqliteConnection,
        track_id: i32,
        reference: Reference,
    ) -> SoundomeResult<Vec<Reference>> {
        self.track_repo
            .create_references(conn, track_id, &[reference])?;
        let track = self.track_repo.get_by_id(conn, track_id)?;
        Ok(track.references)
    }

    /// Delete a single reference row by its own ID.
    pub fn delete_reference(&self, conn: &mut SqliteConnection, ref_id: i32) -> SoundomeResult<()> {
        self.track_repo.delete_reference(conn, ref_id)
    }
}
