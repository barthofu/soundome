-- Generalize sync_schedule from a playlist-only, per-item interval/cron
-- schedule into a generic sync "subscription" list. Scheduling itself is now
-- driven by the single global sync_settings row: subscriptions only carry
-- what to sync and whether they're enabled.

DROP TRIGGER IF EXISTS sync_schedule_check_interval_or_cron;
DROP TRIGGER IF EXISTS sync_schedule_check_interval_or_cron_update;

ALTER TABLE sync_schedule RENAME TO sync_schedule_old;

CREATE TABLE sync_schedule (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- 'playlist' | 'artist'
    entity_type TEXT NOT NULL DEFAULT 'playlist',
    -- set when entity_type = 'artist'
    artist_id INTEGER REFERENCES artist(id) ON DELETE CASCADE,
    -- set when entity_type = 'artist': the specific Source reference chosen
    -- to sync from. Cleared (not cascaded) if the reference is removed, so
    -- the subscription can be surfaced/disabled instead of silently vanishing.
    reference_id INTEGER REFERENCES artist_ref(id) ON DELETE SET NULL,
    -- resolved target URL (playlist source_url, or the artist reference's
    -- external_url), denormalized so the scheduler can enqueue without joins
    url TEXT NOT NULL,
    label TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO sync_schedule (id, entity_type, artist_id, reference_id, url, label, enabled, last_run, created_at)
SELECT id, 'playlist', NULL, NULL, playlist_url, label, enabled, last_run, created_at
FROM sync_schedule_old;

DROP TABLE sync_schedule_old;

CREATE UNIQUE INDEX sync_schedule_artist_reference_uniq
    ON sync_schedule (entity_type, artist_id, reference_id)
    WHERE entity_type = 'artist';

CREATE UNIQUE INDEX sync_schedule_playlist_url_uniq
    ON sync_schedule (entity_type, url)
    WHERE entity_type = 'playlist';
