-- Best-effort revert: restore the old interval/cron-per-item shape.
-- Any per-item scheduling data has been lost (it now lives in sync_settings),
-- so restored rows default to a 1-hour interval.

DROP INDEX IF EXISTS sync_schedule_artist_reference_uniq;
DROP INDEX IF EXISTS sync_schedule_playlist_url_uniq;

ALTER TABLE sync_schedule RENAME TO sync_schedule_new;

CREATE TABLE sync_schedule (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    playlist_url TEXT NOT NULL,
    label TEXT,
    interval_seconds INTEGER,
    cron_expression TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run TIMESTAMP,
    next_run TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO sync_schedule (id, playlist_url, label, interval_seconds, cron_expression, enabled, last_run, next_run, created_at)
SELECT id, url, label, 3600, NULL, enabled, last_run, NULL, created_at
FROM sync_schedule_new
WHERE entity_type = 'playlist';

DROP TABLE sync_schedule_new;

CREATE TRIGGER sync_schedule_check_interval_or_cron
BEFORE INSERT ON sync_schedule
BEGIN
  SELECT CASE
    WHEN NEW.interval_seconds IS NULL AND NEW.cron_expression IS NULL
    THEN RAISE(ABORT, 'At least one of interval_seconds or cron_expression must be set')
  END;
END;

CREATE TRIGGER sync_schedule_check_interval_or_cron_update
BEFORE UPDATE ON sync_schedule
BEGIN
  SELECT CASE
    WHEN NEW.interval_seconds IS NULL AND NEW.cron_expression IS NULL
    THEN RAISE(ABORT, 'At least one of interval_seconds or cron_expression must be set')
  END;
END;
