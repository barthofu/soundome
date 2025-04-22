use diesel::SqliteConnection;
use shared::{models::Artist, types::SoundomeResult};

use crate::{mappers::{convert_artist_entity_to_artist, convert_artist_to_new_artist_entity}, repositories};

pub fn create_artist(conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Artist> {
    let new_artist = convert_artist_to_new_artist_entity(artist);

    let inserted_artist = repositories::artist::create(conn, new_artist)?;

    Ok(convert_artist_entity_to_artist(inserted_artist))
}
