use domain::ports::repositories::PlaylistRepository;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::{models::Playlist, types::SoundomeResult};

use crate::{
    entities::{NewPlaylistEntity, NewPlaylistTrackEntity, PlaylistEntity},
    mappers::map_error,
    schema,
};

pub struct DieselPlaylistRepository {}

impl DieselPlaylistRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl PlaylistRepository for DieselPlaylistRepository {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Playlist> {
        let entity = schema::playlist::table
            .filter(schema::playlist::id.eq(id))
            .first::<PlaylistEntity>(conn)
            .map_err(map_error)?;
        Ok(PlaylistEntity::convert_to_domain(entity))
    }

    fn get_by_source_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<Option<Playlist>> {
        let entity = schema::playlist::table
            .filter(schema::playlist::source_url.eq(url))
            .first::<PlaylistEntity>(conn)
            .optional()
            .map_err(map_error)?;
        Ok(entity.map(PlaylistEntity::convert_to_domain))
    }

    fn create(&self, conn: &mut SqliteConnection, playlist: &Playlist) -> SoundomeResult<Playlist> {
        let new_entity = NewPlaylistEntity::convert_from_domain(playlist);
        diesel::insert_into(schema::playlist::table)
            .values(&new_entity)
            .execute(conn)
            .map_err(map_error)?;
        let created = schema::playlist::table
            .order(schema::playlist::id.desc())
            .first::<PlaylistEntity>(conn)
            .map_err(map_error)?;
        Ok(PlaylistEntity::convert_to_domain(created))
    }

    fn update_last_sync(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        diesel::update(schema::playlist::table.filter(schema::playlist::id.eq(id)))
            .set(schema::playlist::last_sync.eq(Some(chrono::Utc::now().naive_utc())))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn add_track(&self, conn: &mut SqliteConnection, playlist_id: i32, track_id: i32, position: Option<i32>) -> SoundomeResult<()> {
        let entity = NewPlaylistTrackEntity { track_id, playlist_id, position };
        diesel::insert_or_ignore_into(schema::playlist_tracks::table)
            .values(&entity)
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }
}
