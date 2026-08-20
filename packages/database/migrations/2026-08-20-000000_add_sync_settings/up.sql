-- Singleton table holding the single global cron schedule that drives all
-- scheduled sync subscriptions (see sync_schedule). Only one row (id = 1)
-- ever exists.
--
-- The `cron` crate used to compute next_run expects a 6-field expression
-- (sec min hour day month day-of-week), hence '0 0 3 * * *' rather than the
-- more common 5-field '0 3 * * *'.

CREATE TABLE sync_settings (
    id INTEGER NOT NULL PRIMARY KEY CHECK (id = 1),
    cron_expression TEXT NOT NULL DEFAULT '0 0 3 * * *',
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run TIMESTAMP,
    next_run TIMESTAMP
);

INSERT INTO sync_settings (id, cron_expression, enabled, last_run, next_run)
VALUES (1, '0 0 3 * * *', 1, NULL, NULL);
