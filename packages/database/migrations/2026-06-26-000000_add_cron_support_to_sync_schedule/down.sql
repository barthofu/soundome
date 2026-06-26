-- Rollback: remove cron support and make interval_seconds NOT NULL again

DROP TRIGGER IF EXISTS sync_schedule_check_interval_or_cron;
DROP TRIGGER IF EXISTS sync_schedule_check_interval_or_cron_update;

ALTER TABLE sync_schedule RENAME TO sync_schedule_new;

CREATE TABLE sync_schedule (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    playlist_url TEXT NOT NULL,
    label TEXT,
    interval_seconds INTEGER NOT NULL DEFAULT 3600,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run TIMESTAMP,
    next_run TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO sync_schedule (id, playlist_url, label, interval_seconds, enabled, last_run, next_run, created_at)
SELECT id, playlist_url, label, COALESCE(interval_seconds, 3600), enabled, last_run, next_run, created_at
FROM sync_schedule_new;

DROP TABLE sync_schedule_new;
