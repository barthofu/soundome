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

// custom operations

// ================================================================================================
// Album Source
// ================================================================================================

pub mod album_source {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};

    use crate::{
        entities::{AlbumSourceEntity, NewAlbumSourceEntity, UpdateAlbumSourceEntity}, macros
    };

    macros::resource::find_one!(album_source, AlbumSourceEntity);
    macros::resource::find_all!(album_source, AlbumSourceEntity);
    macros::resource::create!(album_source, AlbumSourceEntity, NewAlbumSourceEntity);
    macros::resource::update!(album_source, AlbumSourceEntity, UpdateAlbumSourceEntity);
    macros::resource::delete!(album_source);

}



