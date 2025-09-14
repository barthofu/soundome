use domain::ports::repositories::TrackRepository;

use diesel::prelude::*;
use shared::types::SoundomeResult;

use crate::{
    entities::{AlbumEntity, ArtistEntity, ArtistTrackEntity, NewTrackEntity, TrackEntity, TrackRefEntity, UpdateTrackEntity, NewTrackRefEntity}, macros, schema,
};

pub struct DieselTrackRepository {}

impl DieselTrackRepository {
    pub fn new() -> Self {
        Self { }
    }
}

impl TrackRepository for DieselTrackRepository {

    // =================================================================================
    // CRUD
    // =================================================================================

    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<shared::models::Track> {
        let (track, album): (TrackEntity, Option<AlbumEntity>) = schema::track::table
            .left_join(schema::album::table.on(schema::album::id.nullable().eq(schema::track::album_id)))
            .filter(schema::track::id.eq(id))
            .first(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let artists: Vec<ArtistEntity> = schema::artist_tracks::table
            .inner_join(schema::artist::table.on(schema::artist_tracks::artist_id.eq(schema::artist::id)))
            .filter(schema::artist_tracks::track_id.eq(track.id))
            .select(schema::artist::all_columns)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let references: Vec<TrackRefEntity> = schema::track_ref::table
            .filter(schema::track_ref::track_id.eq(track.id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;
        
        Ok(TrackEntity::convert_to_domain(
            track,
            album,
            artists,
            references,
        ))
    }

    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<shared::models::Track>> {
        let tracks: Vec<TrackEntity> = schema::track::table
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get all resources: {}",
                    err
                ))
            })?;

        let mut result = Vec::new();
        for track in tracks {
            let album = get_album(conn, &track);
            let artists: Vec<ArtistEntity> = schema::artist_tracks::table
                .inner_join(schema::artist::table.on(schema::artist_tracks::artist_id.eq(schema::artist::id)))
                .filter(schema::artist_tracks::track_id.eq(track.id))
                .select(schema::artist::all_columns)
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get all resources: {}",
                        err
                    ))
                })?;

            let references: Vec<TrackRefEntity> = schema::track_ref::table
                .filter(schema::track_ref::track_id.eq(track.id))
                .load(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to get all resources: {}",
                        err
                    ))
                })?;

            result.push(TrackEntity::convert_to_domain(
                track,
                album,
                artists,
                references,
            ));
        }

        Ok(result)
    }

    fn create(&self, conn: &mut SqliteConnection, new_track: &shared::models::Track) -> SoundomeResult<shared::models::Track> {
        let new_track_entity = NewTrackEntity::convert_from_domain(new_track);
        let inserted_track = diesel::insert_into(schema::track::table)
            .values(&new_track_entity)
            .execute(conn)
            .and_then(|_| {
                schema::track::table
                    .order(schema::track::id.desc())
                    .first::<TrackEntity>(conn)
            })
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to create resource: {}",
                    err
                ))
            })?;

        Ok(TrackEntity::convert_to_domain(
            inserted_track,
            None,
            vec![],
            vec![],
        ))
    }

    fn update(&self, conn: &mut SqliteConnection, id: i32, updated_track: &shared::models::Track) -> SoundomeResult<shared::models::Track> {
        let updated_track_entity = UpdateTrackEntity::convert_from_domain(updated_track);
        let updated_track = diesel::update(schema::track::table.filter(schema::track::id.eq(id)))
            .set(&updated_track_entity)
            .execute(conn)
            .and_then(|_| {
                schema::track::table
                    .filter(schema::track::id.eq(id))
                    .first::<TrackEntity>(conn)
            })
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to update resource: {}",
                    err
                ))
            })?;

        Ok(TrackEntity::convert_to_domain(
            updated_track,
            None,
            vec![],
            vec![],
        ))
    }

    // =================================================================================
    // Custom
    // =================================================================================

    fn get_by_url(&self, conn: &mut SqliteConnection, url: &str) -> SoundomeResult<shared::models::Track> {
        let track_ref = schema::track_ref::table
            .filter(schema::track_ref::external_url.eq(url))
            .first::<TrackRefEntity>(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by url: {}",
                    err
                ))
            })?;

        self.get_by_id(conn, track_ref.track_id)
    }

    fn create_references(&self, conn: &mut SqliteConnection, track_id: i32, references: &[shared::models::Reference]) -> SoundomeResult<()> {
        for reference in references {
            let new_track_ref = NewTrackRefEntity::convert_from_domain(reference, track_id);
            
            diesel::insert_into(schema::track_ref::table)
                .values(&new_track_ref)
                .execute(conn)
                .map_err(|err| {
                    shared::errors::Error::Database(format!(
                        "Failed to create track reference: {}",
                        err
                    ))
                })?;
        }
        Ok(())
    }
}













// ================================================================================================
// ARCHIVES
// ================================================================================================

// basic CRUD operations

macros::resource::find_one!(track, TrackEntity);
macros::resource::find_all!(track, TrackEntity);
macros::resource::create!(track, TrackEntity, NewTrackEntity);
macros::resource::update!(track, TrackEntity, UpdateTrackEntity);
macros::resource::delete!(track);

// associations

macros::association::many_to_many::get_all_associations!(
    get_artists,
    TrackEntity,
    artist,
    ArtistEntity,
    artist_tracks,
    ArtistTrackEntity,
    artist_id,
);

macros::association::many_to_many::create_association!(
    create_artist,
    TrackEntity,
    ArtistEntity,
    artist_tracks,
    ArtistTrackEntity,
    track_id,
    artist_id,
);

macros::association::many_to_many::delete_association!(
    delete_artist,
    TrackEntity,
    ArtistEntity,
    artist_tracks,
    track_id,
    artist_id,
);

pub fn get_album(conn: &mut SqliteConnection, track: &TrackEntity) -> Option<AlbumEntity> {

    match track.album_id {
        Some(album_id) => super::album::find_one(conn, album_id).ok(),
        None => None,
    }
}

// ================================================================================================
// Track Ref
// ================================================================================================

pub mod track_ref {
    use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

    use crate::{
        entities::{NewTrackRefEntity, TrackEntity, TrackRefEntity, UpdateTrackRefEntity}, macros
    };

    macros::resource::find_one!(track_ref, TrackRefEntity);
    macros::resource::find_all!(track_ref, TrackRefEntity);
    macros::resource::create!(track_ref, TrackRefEntity, NewTrackRefEntity);
    macros::resource::update!(track_ref, TrackRefEntity, UpdateTrackRefEntity);
    macros::resource::delete!(track_ref);

    // custom operations

    pub fn get_track_refs_by_track_entity(
        conn: &mut SqliteConnection,
        track: &TrackEntity,
    ) -> Vec<TrackRefEntity> {
        TrackRefEntity::belonging_to(track)
            .load::<TrackRefEntity>(conn)
            .unwrap_or_default()
    }
}
