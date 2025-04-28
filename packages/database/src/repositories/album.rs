// basic CRUD operations

use core::ports::repositories::AlbumRepository;

use diesel::{BelongingToDsl, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::types::SoundomeResult;

use crate::{
    entities::{
        AlbumEntity, AlbumRefEntity, ArtistAlbumEntity, ArtistEntity, NewAlbumEntity, UpdateAlbumEntity
    }, macros, schema
};

pub struct DieselAlbumRepository {}

impl AlbumRepository for DieselAlbumRepository {
    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<shared::models::Album> {
        let album: AlbumEntity = schema::album::table
            .filter(schema::album::id.eq(id))
            .first(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let artists: Vec<ArtistEntity> = schema::artist_albums::table
            .inner_join(schema::artist::table.on(schema::artist_albums::artist_id.eq(schema::artist::id)))
            .filter(schema::artist_albums::album_id.eq(album.id))
            .select(schema::artist::all_columns)
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        let references: Vec<AlbumRefEntity> = schema::album_ref::table
            .filter(schema::album_ref::album_id.eq(album.id))
            .load(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by id: {}",
                    err
                ))
            })?;

        Ok(AlbumEntity::convert_to_domain(
            album,
            artists,
            references,
        ))
    }

    fn create(
        &self,
        conn: &mut SqliteConnection,
        new_album: &shared::models::Album,
    ) -> SoundomeResult<shared::models::Album> {

        let new_album_entity = NewAlbumEntity::convert_from_domain(new_album);
        let inserted_album = diesel::insert_into(schema::album::table)
            .values(&new_album_entity)
            .execute(conn)
            .and_then(|_| {
                schema::album::table
                    .order(schema::album::id.desc())
                    .first::<AlbumEntity>(conn)
            })
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to create resource: {}",
                    err
                ))
            })?;

        Ok(AlbumEntity::convert_to_domain(
            inserted_album,
            vec![],
            vec![],
        ))
    }

    fn update(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        updated_album: &shared::models::Album,
    ) -> SoundomeResult<shared::models::Album> {
        let updated_album_entity = UpdateAlbumEntity::convert_from_domain(updated_album);
        diesel::update(schema::album::table)
            .filter(schema::album::id.eq(id))
            .set(&updated_album_entity)
            .execute(conn)
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to update resource: {}",
                    err
                ))
            })?;

        self.get_by_id(conn, id)
    }
}

























// ================================================================================================
// ARCHIVES
// ================================================================================================


macros::resource::find_one!(album, AlbumEntity);
macros::resource::find_all!(album, AlbumEntity);
macros::resource::create!(album, AlbumEntity, NewAlbumEntity);
macros::resource::update!(album, AlbumEntity, UpdateAlbumEntity);
macros::resource::delete!(album);

// associations

macros::association::many_to_many::get_all_associations!(
    get_artists,
    AlbumEntity,
    artist,
    ArtistEntity,
    artist_albums,
    ArtistAlbumEntity,
    artist_id,
);

macros::association::many_to_many::create_association!(
    create_artist,
    AlbumEntity,
    ArtistEntity,
    artist_albums,
    ArtistAlbumEntity,
    album_id,
    artist_id,
);

macros::association::many_to_many::delete_association!(
    delete_artist,
    AlbumEntity,
    ArtistEntity,
    artist_albums,
    album_id,
    artist_id,
);

// ================================================================================================
// Album Ref
// ================================================================================================

pub mod album_ref {
    use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

    use crate::{
        entities::{AlbumEntity, AlbumRefEntity, NewAlbumRefEntity, UpdateAlbumRefEntity}, macros
    };

    macros::resource::find_one!(album_ref, AlbumRefEntity);
    macros::resource::find_all!(album_ref, AlbumRefEntity);
    macros::resource::create!(album_ref, AlbumRefEntity, NewAlbumRefEntity);
    macros::resource::update!(album_ref, AlbumRefEntity, UpdateAlbumRefEntity);
    macros::resource::delete!(album_ref);

    // custom operations

    pub fn get_album_refs_by_album_entity(
        conn: &mut SqliteConnection,
        album_entity: &AlbumEntity,
    ) -> Vec<AlbumRefEntity> {
        AlbumRefEntity::belonging_to(album_entity)
            .load::<AlbumRefEntity>(conn)
            .unwrap_or_default()
    }
}

