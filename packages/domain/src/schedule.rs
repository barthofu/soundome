use chrono::NaiveDateTime;
use shared::errors::Error;
use shared::models::SyncEntityType;
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
}
