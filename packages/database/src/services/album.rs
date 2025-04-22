use diesel::SqliteConnection;
use shared::{models::Album, types::SoundomeResult};

use crate::{mappers::{convert_album_entity_to_track, convert_album_to_new_album_entity, convert_artist_to_new_artist_entity}, repositories};

pub fn create_album(conn: &mut SqliteConnection, album: &Album) -> SoundomeResult<Album> {
    let new_album = convert_album_to_new_album_entity(album);

    // create album
    let inserted_album = repositories::album::create(conn, new_album)?;

    // create album artists
    let artists = album.artists.iter().map(|artist| {
        let new_artist = convert_artist_to_new_artist_entity(artist);
        let inserted_artist = repositories::artist::create(conn, new_artist)?;
        Ok(inserted_artist)
    }).collect::<SoundomeResult<Vec<_>>>()?;

    // create relationships
    for artist in artists {
        repositories::album::create_artist(conn, &inserted_album, &artist)?;
    }

    convert_album_entity_to_track(conn, inserted_album)
}
