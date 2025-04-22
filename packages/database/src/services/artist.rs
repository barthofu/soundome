use diesel::SqliteConnection;
use shared::{models::Artist, types::SoundomeResult};

use crate::{mappers::{convert_artist_entity_to_artist, convert_artist_ref_to_new_artist_ref_entity, convert_artist_to_new_artist_entity}, repositories};

pub fn create_artist(conn: &mut SqliteConnection, artist: &Artist) -> SoundomeResult<Artist> {
    let new_artist = convert_artist_to_new_artist_entity(artist);

    // create artist
    let inserted_artist = repositories::artist::create(conn, new_artist)?;

    // create artist references
    for reference in &artist.references {
        let new_reference = convert_artist_ref_to_new_artist_ref_entity(reference, inserted_artist.id);
        repositories::artist::artist_ref::create(conn, new_reference)?;
    }
        
    Ok(convert_artist_entity_to_artist(conn, inserted_artist))
}
