use diesel::SqliteConnection;
use shared::{models::Track, types::SoundomeResult};

use crate::{mappers::{convert_album_to_new_album_entity, convert_artist_to_new_artist_entity, convert_track_entity_to_track, convert_track_ref_to_new_track_ref_entity, convert_track_to_new_track_entity}, repositories};

pub fn create_track(conn: &mut SqliteConnection, track: &Track) -> SoundomeResult<Track> {
    // create album if it exists
    let album_id = track.album.as_ref()
        .map(|album| {
            let new_album = convert_album_to_new_album_entity(album);
            let inserted_album = repositories::album::create(conn, new_album)?;
            Ok::<_, shared::errors::Error>(inserted_album.id)
        })
        .transpose()?;

    // create track
    let new_track = convert_track_to_new_track_entity(track, album_id);
    let inserted_track = repositories::track::create(conn, new_track)?;

    // create track references
    for reference in &track.references {
        let new_reference = convert_track_ref_to_new_track_ref_entity(reference, inserted_track.id);
        repositories::track::track_ref::create(conn, new_reference)?;
    }

    // create track artists
    let artists = track.artists.iter().map(|artist| {
        let new_artist = convert_artist_to_new_artist_entity(artist);
        let inserted_artist = repositories::artist::create(conn, new_artist)?;
        Ok(inserted_artist)
    }).collect::<SoundomeResult<Vec<_>>>()?;

    // create relationships
    for artist in artists {
        repositories::track::create_artist(conn, &inserted_track, &artist)?;
    }

    convert_track_entity_to_track(conn, inserted_track)
}
