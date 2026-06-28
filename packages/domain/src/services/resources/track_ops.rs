use std::sync::Arc;

use diesel::SqliteConnection;
use shared::types::SoundomeResult;
use tracing::debug;

use crate::ports::repositories::{AlbumRepository, ArtistRepository, TrackRepository};

/// Delete a single track and cascade-delete its album/artists if they become orphans.
///
/// This is the canonical delete path used by both `TrackService` and `PlaylistService`
/// so that orphan cleanup always runs regardless of the call site.
pub fn delete_track_with_cascade(
    conn: &mut SqliteConnection,
    track_id: i32,
    track_repo: &Arc<dyn TrackRepository + Send + Sync>,
    album_repo: &Arc<dyn AlbumRepository + Send + Sync>,
    artist_repo: &Arc<dyn ArtistRepository + Send + Sync>,
) -> SoundomeResult<()> {
    use diesel::Connection as _;

    conn.transaction(|tx| {
        // 1) Load the track first so we know which album/artists to check after deletion.
        let track = track_repo.get_by_id(tx, track_id)?;

        let album_id = track.album.as_ref().and_then(|a| a.id);
        let artist_ids: Vec<i32> = track.artists.iter().filter_map(|a| a.id).collect();

        // 2) Delete the track row (also removes artist_tracks / track_ref via delete_with_relations!).
        track_repo.delete(tx, track_id)?;

        // 3) Orphan check — album.
        if let Some(aid) = album_id {
            let remaining = album_repo.count_tracks(tx, aid)?;
            if remaining == 0 {
                debug!(
                    album_id = aid,
                    "album has no remaining tracks after track deletion, removing it"
                );
                album_repo.delete(tx, aid)?;
            }
        }

        // 4) Orphan check — artists.
        for aid in artist_ids {
            let remaining = artist_repo.count_tracks(tx, aid)?;
            if remaining == 0 {
                debug!(
                    artist_id = aid,
                    "artist has no remaining tracks after track deletion, removing it"
                );
                artist_repo.delete(tx, aid)?;
            }
        }

        Ok(())
    })
}
