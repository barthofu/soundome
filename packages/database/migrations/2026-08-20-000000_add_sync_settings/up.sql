-- Singleton table holding the single global cron schedule that drives all
-- scheduled sync subscriptions (see sync_schedule). Only one row (id = 1)
-- ever exists.

CREATE TABLE sync_settings (
    id INTEGER NOT NULL PRIMARY KEY CHECK (id = 1),
    cron_expression TEXT NOT NULL DEFAULT '0 3 * * *',
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run TIMESTAMP,
    next_run TIMESTAMP
);

INSERT INTO sync_settings (id, cron_expression, enabled, last_run, next_run)
VALUES (1, '0 3 * * *', 1, NULL, NULL);
