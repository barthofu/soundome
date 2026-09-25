use std::{collections::HashSet, sync::Arc};

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

    /// Paginated, searched, sorted, filtered listing for the library UI.
    pub fn get_page(
        &self,
        conn: &mut SqliteConnection,
        query: crate::ports::repositories::TrackQuery,
    ) -> SoundomeResult<crate::ports::repositories::Page<Track>> {
        self.track_repo.get_page(conn, query)
    }

    /// All tracks linked to a given artist (artist drill-down view).
    pub fn get_by_artist(
        &self,
        conn: &mut SqliteConnection,
        artist_id: i32,
    ) -> SoundomeResult<Vec<Track>> {
        self.track_repo.get_by_artist(conn, artist_id)
    }

    /// All tracks linked to a given album (album drill-down view).
    pub fn get_by_album(
        &self,
        conn: &mut SqliteConnection,
        album_id: i32,
    ) -> SoundomeResult<Vec<Track>> {
        self.track_repo.get_by_album(conn, album_id)
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

    /// Delete a pending-validation track and its staged audio file.
    ///
    /// The file is removed before the database row so a filesystem failure
    /// leaves the validation row available for retry instead of orphaning the
    /// staged audio file.
    pub fn delete_pending_validation(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> SoundomeResult<()> {
        let track = self.track_repo.get_by_id(conn, id)?;

        if !track.needs_validation {
            return Err(Error::Custom(format!(
                "track {} is not pending validation",
                id
            )));
        }

        if let Some(file_path) = track.file_path {
            match std::fs::remove_file(&file_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(Error::Custom(format!(
                        "Failed to delete staged audio file {:?}: {}",
                        file_path, error
                    )));
                }
            }
        }

        self.delete_by_id(conn, id)
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

    /// Merge duplicate finalized tracks into one survivor. The recording
    /// selected by the existing quality comparator used during download-time deduplication
    /// supplies the surviving metadata and audio path. Playlist memberships
    /// and durable metadata references from every duplicate are preserved.
    pub fn merge_into(
        &self,
        conn: &mut SqliteConnection,
        source_ids: &[i32],
        target_id: i32,
    ) -> SoundomeResult<Track> {
        if source_ids.is_empty() || source_ids.contains(&target_id) {
            return Err(Error::Custom(
                "Track merge requires source ids distinct from the target".to_string(),
            ));
        }
        let mut seen = HashSet::with_capacity(source_ids.len());
        if source_ids.iter().any(|id| !seen.insert(*id)) {
            return Err(Error::Custom(
                "Track merge source ids must be unique".to_string(),
            ));
        }

        let target = self.track_repo.get_by_id(conn, target_id)?;
        let mut tracks = Vec::with_capacity(source_ids.len() + 1);
        tracks.push(target);
        for source_id in source_ids {
            let source = self.track_repo.get_by_id(conn, *source_id)?;
            if source.needs_validation || source.file_path.is_none() {
                return Err(Error::Custom(format!(
                    "Track {} is not finalized and cannot be merged from the duplicate queue",
                    source_id
                )));
            }
            tracks.push(source);
        }
        if tracks[0].needs_validation || tracks[0].file_path.is_none() {
            return Err(Error::Custom(format!(
                "Track {} is not finalized and cannot be merged from the duplicate queue",
                target_id
            )));
        }

        // Keep the requested target row id, but select the actual best-quality
        // recording/metadata as the content for that survivor.
        let mut winner = tracks[0].clone();
        for track in tracks.iter().skip(1) {
            if self.is_better_quality(&winner, track) {
                winner = track.clone();
            }
        }

        let winner_id = winner.id;
        let winner_path = winner.file_path.clone();
        let mut merged_references = Vec::new();
        for track in &tracks {
            let is_winner = track.id == winner_id;
            for reference in &track.references {
                // Source/Provider describe the selected audio path. Keep those
                // only from the quality winner; Metadata/Reference are durable
                // and are merged from all duplicate rows.
                if !is_winner
                    && (reference.ref_type == ReferenceType::Source
                        || reference.ref_type == ReferenceType::Provider)
                {
                    continue;
                }
                let already_present = merged_references.iter().any(|existing: &Reference| {
                    existing.ref_type == reference.ref_type
                        && existing.platform == reference.platform
                        && existing.external_id == reference.external_id
                        && existing.external_url == reference.external_url
                });
                if !already_present {
                    let mut reference = reference.clone();
                    reference.id = None;
                    merged_references.push(reference);
                }
            }
        }

        // A reference setter replaces Source/Provider only when it receives an
        // entry of that type. If the selected recording has no reference of a
        // type that an inferior duplicate did have, pass an empty sentinel so
        // the stale audio-path reference is cleared rather than left on target.
        for ref_type in [ReferenceType::Source, ReferenceType::Provider] {
            let winner_has_type = winner
                .references
                .iter()
                .any(|reference| reference.ref_type == ref_type);
            let duplicate_had_type = tracks.iter().any(|track| {
                track
                    .references
                    .iter()
                    .any(|reference| reference.ref_type == ref_type)
            });
            if !winner_has_type && duplicate_had_type {
                if let Some(mut sentinel) = tracks
                    .iter()
                    .flat_map(|track| track.references.iter())
                    .find(|reference| reference.ref_type == ref_type)
                    .cloned()
                {
                    sentinel.id = None;
                    sentinel.external_id = None;
                    sentinel.external_url = None;
                    merged_references.push(sentinel);
                }
            }
        }

        let mut merged_track = winner.clone();
        merged_track.id = Some(target_id);
        merged_track.references = merged_references;
        let losing_paths: Vec<_> = tracks
            .iter()
            .filter(|track| track.id != winner_id)
            .filter_map(|track| track.file_path.clone())
            .filter(|path| Some(path) != winner_path.as_ref())
            .collect();

        self.track_repo
            .merge_into(conn, source_ids, target_id, &merged_track)?;

        // Database state is committed at this point. File cleanup is best
        // effort: a filesystem error must not undo or obscure the successful DB
        // merge; it is logged so an orphan file can be cleaned up later.
        let mut removed = HashSet::new();
        for path in losing_paths {
            if !removed.insert(path.clone()) {
                continue;
            }
            match std::fs::remove_file(&path) {
                Ok(()) => tracing::info!(?path, "Removed inferior duplicate track audio"),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    tracing::warn!(?path, %error, "Could not remove inferior duplicate track audio")
                }
            }
        }

        self.track_repo.get_by_id(conn, target_id)
    }

    /// Delete track file
    pub fn delete_track_file(&self, track: &Track) -> SoundomeResult<bool> {
        let file_deleted = if let Some(file_path) = &track.file_path {
            match std::fs::remove_file(file_path) {
                Ok(()) => true,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => {
                    return Err(Error::Custom(format!(
                        "Failed to delete audio file {:?}: {}",
                        file_path, error
                    )));
                }
            }
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
