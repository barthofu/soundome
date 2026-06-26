use chrono::NaiveDateTime;
use shared::errors::Error;
use shared::types::SoundomeResult;

/// Calculate the next run time based on either interval_seconds or cron_expression.
pub fn calculate_next_run(
    now: NaiveDateTime,
    interval_seconds: Option<i32>,
    cron_expression: Option<&str>,
) -> SoundomeResult<NaiveDateTime> {
    match (interval_seconds, cron_expression) {
        (Some(interval), _) if interval > 0 => {
            // Use interval-based scheduling
            Ok(now + chrono::Duration::seconds(interval as i64))
        }
        (_, Some(cron_expr)) => {
            // Use cron-based scheduling
            calculate_next_run_from_cron(now, cron_expr)
        }
        _ => Err(Error::InvalidArg),
    }
}

/// Calculate next run time from a cron expression.
fn calculate_next_run_from_cron(
    now: NaiveDateTime,
    cron_expr: &str,
) -> SoundomeResult<NaiveDateTime> {
    use cron::Schedule;
    use std::str::FromStr;

    let schedule = Schedule::from_str(cron_expr).map_err(|_| Error::InvalidArg)?;

    // Convert NaiveDateTime to chrono::DateTime in UTC
    let dt = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(now, chrono::Utc);

    // Get the next occurrence after the given datetime
    let next_dt = schedule
        .after(&dt)
        .next()
        .ok_or(Error::Internal(
            "Could not calculate next occurrence from cron expression".to_string(),
        ))?;

    Ok(next_dt.naive_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_next_run_with_interval() {
        let now = chrono::NaiveDateTime::from_timestamp_opt(1000000, 0).unwrap();
        let next = calculate_next_run(now, Some(3600), None).unwrap();
        let expected = now + chrono::Duration::seconds(3600);
        assert_eq!(next, expected);
    }

    #[test]
    fn test_calculate_next_run_with_cron() {
        let now = chrono::NaiveDateTime::from_timestamp_opt(1000000, 0).unwrap();
        // "0 12 * * *" means at 12:00 every day
        let result = calculate_next_run(now, None, Some("0 12 * * *"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_next_run_missing_both() {
        let now = chrono::NaiveDateTime::from_timestamp_opt(1000000, 0).unwrap();
        let result = calculate_next_run(now, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_next_run_invalid_cron() {
        let now = chrono::NaiveDateTime::from_timestamp_opt(1000000, 0).unwrap();
        let result = calculate_next_run(now, None, Some("invalid cron"));
        assert!(result.is_err());
    }
}
