// basic CRUD operations

use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

use crate::{
    entities::{
        AlbumEntity, ArtistAlbumEntity, ArtistEntity, NewAlbumEntity, UpdateAlbumEntity
    }, macros
};

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

