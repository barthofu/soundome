use std::sync::Arc;

use diesel::SqliteConnection;
use shared::{models::Reference, types::SoundomeResult};

use crate::ports::repositories::{AlbumRepository, ArtistRepository};

pub struct AlbumService {
    album_repo: Arc<dyn AlbumRepository + Send + Sync>,
    #[allow(dead_code)]
    artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
}

impl AlbumService {
    pub fn new(
        album_repo: Arc<dyn AlbumRepository + Send + Sync>,
        artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
    ) -> Self {
        Self {
            album_repo,
            artist_repo,
        }
    }

    // CRUD

    pub fn get_by_id(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> SoundomeResult<shared::models::Album> {
        self.album_repo.get_by_id(conn, id)
    }

    pub fn get_all(
        &self,
        conn: &mut SqliteConnection,
    ) -> SoundomeResult<Vec<shared::models::Album>> {
        self.album_repo.get_all(conn)
    }

    pub fn create(
        &self,
        conn: &mut SqliteConnection,
        new_album: &shared::models::Album,
    ) -> SoundomeResult<shared::models::Album> {
        self.album_repo.create(conn, new_album)
    }

    pub fn update(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        updated_album: &shared::models::Album,
    ) -> SoundomeResult<shared::models::Album> {
        self.album_repo.update(conn, id, updated_album)
    }

    pub fn delete_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        self.album_repo.delete(conn, id)
    }

    // Getters

    pub fn get_by_url(
        &self,
        conn: &mut SqliteConnection,
        url: &str,
    ) -> Option<shared::models::Album> {
        self.album_repo.get_by_url(conn, url).ok()
    }

    pub fn count(&self, conn: &mut SqliteConnection) -> SoundomeResult<i64> {
        self.album_repo.count(conn)
    }

    /// Append a single reference to an album and return the full updated list.
    pub fn add_reference(
        &self,
        conn: &mut SqliteConnection,
        album_id: i32,
        reference: Reference,
    ) -> SoundomeResult<Vec<Reference>> {
        self.album_repo.create_references(conn, album_id, &[reference])?;
        let album = self.album_repo.get_by_id(conn, album_id)?;
        Ok(album.references)
    }

    /// Delete a single reference row by its own ID.
    pub fn delete_reference(
        &self,
        conn: &mut SqliteConnection,
        ref_id: i32,
    ) -> SoundomeResult<()> {
        self.album_repo.delete_reference(conn, ref_id)
    }

    // Custom

    #[allow(dead_code)]
    fn create_or_ignore(
        &self,
        conn: &mut SqliteConnection,
        album: &shared::models::Album,
    ) -> SoundomeResult<shared::models::Album> {
        // Step 1: Use create_or_ignore for the album
        let created_album = self.album_repo.create_or_ignore(conn, album)?;
        let album_id = created_album.id.unwrap();

        // Step 2: Handle album artists using create_or_ignore
        for artist in &album.artists {
            let created_artist = self.artist_repo.create_or_ignore(conn, artist)?;
            let artist_id = created_artist.id.unwrap();

            // Create artist-album relationship
            self.artist_repo
                .create_album_relationship(conn, artist_id, album_id)?;
        }

        // Step 3: Load the complete album with all relationships for return
        self.album_repo.get_by_id(conn, album_id)
    }
}
