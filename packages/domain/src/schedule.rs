use chrono::NaiveDateTime;
use shared::errors::Error;
use shared::models::{Platform, Reference, ReferenceType, SyncEntityType};
use shared::types::SoundomeResult;

/// Best-effort URL-type detection for the manual "add link" scheduled-sync
/// flow: reuses the same URL-shape checks the fetcher already applies when
/// resolving a source. Falls back to `Playlist` when the URL isn't
/// recognized as an artist link (playlist/track URLs are handled the same
/// way today, and album support is intentionally out of scope for now).
pub fn detect_sync_entity_type(url: &str) -> SyncEntityType {
    use fetcher::Source;
    if fetcher::Fetcher::is_valid_artist_url(url) {
        SyncEntityType::Artist
    } else {
        SyncEntityType::Playlist
    }
}

/// Whether an artist reference can be used as a scheduled-sync target.
///
/// `packages/fetcher` only tags YouTube Music artists with
/// `ReferenceType::Source` — Spotify and SoundCloud artists carry their
/// durable id as `ReferenceType::Metadata` instead (see
/// `packages/fetcher/src/{spotify,soundcloud}/mappers.rs`). So `Metadata`
/// references must also be accepted, but only for platforms that are
/// actually a valid sync "source" for an artist (i.e. ones `Fetcher` can
/// resolve via `get_artist_from_url`) — a `Metadata` reference to
/// MusicBrainz, for instance, is enrichment-only and can never be synced
/// from directly.
pub fn is_eligible_artist_sync_reference(reference: &Reference) -> bool {
    if reference.external_url.is_none() {
        return false;
    }
    match reference.ref_type {
        ReferenceType::Source => true,
        ReferenceType::Metadata => matches!(
            reference.platform,
            Platform::Spotify | Platform::SoundCloud | Platform::YoutubeMusic
        ),
        ReferenceType::Provider | ReferenceType::Reference => false,
    }
}

/// Calculate the next run time from a cron expression.
///
/// Scheduled sync only supports a single global cron expression (see
/// `SyncSettings`); per-item interval-based scheduling has been removed.
pub fn calculate_next_run(
    now: NaiveDateTime,
    cron_expression: &str,
) -> SoundomeResult<NaiveDateTime> {
    use cron::Schedule;
    use std::str::FromStr;

    let schedule = Schedule::from_str(cron_expression).map_err(|_| Error::InvalidArg)?;

    // Convert NaiveDateTime to chrono::DateTime in UTC
    let dt = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(now, chrono::Utc);

    // Get the next occurrence after the given datetime
    let next_dt = schedule.after(&dt).next().ok_or(Error::Internal(
        "Could not calculate next occurrence from cron expression".to_string(),
    ))?;

    Ok(next_dt.naive_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_next_run_with_cron() {
        let now = chrono::DateTime::<chrono::Utc>::from_timestamp(1000000, 0)
            .unwrap()
            .naive_utc();
        // "0 0 12 * * *" means at 12:00 every day (6-field cron format: second minute hour day month dayofweek)
        let result = calculate_next_run(now, "0 0 12 * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_next_run_invalid_cron() {
        let now = chrono::DateTime::<chrono::Utc>::from_timestamp(1000000, 0)
            .unwrap()
            .naive_utc();
        let result = calculate_next_run(now, "invalid cron");
        assert!(result.is_err());
    }

    fn reference(ref_type: ReferenceType, platform: Platform) -> Reference {
        Reference {
            id: Some(1),
            ref_type,
            platform,
            external_id: None,
            external_url: Some("https://example.com".to_string()),
        }
    }

    #[test]
    fn source_reference_is_always_eligible() {
        assert!(is_eligible_artist_sync_reference(&reference(
            ReferenceType::Source,
            Platform::YoutubeMusic
        )));
    }

    #[test]
    fn metadata_reference_eligible_for_spotify_soundcloud_youtubemusic() {
        for platform in [
            Platform::Spotify,
            Platform::SoundCloud,
            Platform::YoutubeMusic,
        ] {
            assert!(is_eligible_artist_sync_reference(&reference(
                ReferenceType::Metadata,
                platform
            )));
        }
    }

    #[test]
    fn metadata_reference_ineligible_for_non_source_platforms() {
        for platform in [
            Platform::MusicBrainz,
            Platform::Youtube,
            Platform::Bandcamp,
            Platform::Unknown,
        ] {
            assert!(!is_eligible_artist_sync_reference(&reference(
                ReferenceType::Metadata,
                platform
            )));
        }
    }

    #[test]
    fn provider_and_reference_types_are_ineligible() {
        assert!(!is_eligible_artist_sync_reference(&reference(
            ReferenceType::Provider,
            Platform::Spotify
        )));
        assert!(!is_eligible_artist_sync_reference(&reference(
            ReferenceType::Reference,
            Platform::Spotify
        )));
    }

    #[test]
    fn reference_without_url_is_ineligible() {
        let mut r = reference(ReferenceType::Metadata, Platform::Spotify);
        r.external_url = None;
        assert!(!is_eligible_artist_sync_reference(&r));
    }
}
