use domain::ports::repositories::PlaylistRepository;

use diesel::prelude::*;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::{models::Playlist, types::SoundomeResult};

use crate::{
    entities::{
        AlbumEntity, ArtistEntity, NewPlaylistEntity, NewPlaylistTrackEntity, PlaylistEntity,
        TrackEntity, TrackRefEntity,
    },
    mappers::map_error,
    schema,
};

#[derive(Default)]
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

    fn get_by_source_url(
        &self,
        conn: &mut SqliteConnection,
        url: &str,
    ) -> SoundomeResult<Option<Playlist>> {
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

    fn add_track(
        &self,
        conn: &mut SqliteConnection,
        playlist_id: i32,
        track_id: i32,
        position: Option<i32>,
    ) -> SoundomeResult<()> {
        let entity = NewPlaylistTrackEntity {
            track_id,
            playlist_id,
            position,
        };
        diesel::insert_or_ignore_into(schema::playlist_tracks::table)
            .values(&entity)
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn get_tracks(
        &self,
        conn: &mut SqliteConnection,
        playlist_id: i32,
    ) -> SoundomeResult<Vec<shared::models::Track>> {
        // Load track entities joined via the junction table, ordered by position.
        let track_entities: Vec<TrackEntity> = schema::playlist_tracks::table
            .inner_join(
                schema::track::table.on(schema::playlist_tracks::track_id.eq(schema::track::id)),
            )
            .filter(schema::playlist_tracks::playlist_id.eq(playlist_id))
            .order(schema::playlist_tracks::position.asc())
            .select(schema::track::all_columns)
            .load(conn)
            .map_err(|e| {
                shared::errors::Error::Database(format!(
                    "Failed to load tracks for playlist {}: {}",
                    playlist_id, e
                ))
            })?;

        let mut result = Vec::with_capacity(track_entities.len());
        for track in track_entities {
            let album = if let Some(album_id) = track.album_id {
                schema::album::table
                    .filter(schema::album::id.eq(album_id))
                    .first::<AlbumEntity>(conn)
                    .ok()
            } else {
                None
            };

            let artists: Vec<ArtistEntity> = schema::artist_tracks::table
                .inner_join(
                    schema::artist::table
                        .on(schema::artist_tracks::artist_id.eq(schema::artist::id)),
                )
                .filter(schema::artist_tracks::track_id.eq(track.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|e| {
                    shared::errors::Error::Database(format!(
                        "Failed to load artists for track {}: {}",
                        track.id, e
                    ))
                })?;

            let references: Vec<TrackRefEntity> = schema::track_ref::table
                .filter(schema::track_ref::track_id.eq(track.id))
                .load(conn)
                .map_err(|e| {
                    shared::errors::Error::Database(format!(
                        "Failed to load references for track {}: {}",
                        track.id, e
                    ))
                })?;

            result.push(TrackEntity::convert_to_domain(
                track, album, artists, references,
            ));
        }

        Ok(result)
    }

    fn count(&self, conn: &mut SqliteConnection) -> SoundomeResult<i64> {
        schema::playlist::table
            .count()
            .get_result(conn)
            .map_err(map_error)
    }
}
