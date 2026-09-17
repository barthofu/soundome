-- Audit trail of every metadata change applied by the AI cleanup step
-- (packages/fetcher/src/soundcloud/mod.rs). Purely additive/diagnostic: lets
-- Data Quality > Cleanup surface exactly what the AI changed so mistakes
-- (e.g. an invented artist name) can be spotted without waiting for a user
-- to notice a wrong track in the library.

CREATE TABLE ai_cleanup_log (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER,
    platform TEXT NOT NULL,
    source_external_id TEXT,
    before_title TEXT NOT NULL,
    before_artists TEXT NOT NULL,
    after_title TEXT NOT NULL,
    after_artists TEXT NOT NULL,
    rejected_artists TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ai_cleanup_log_track ON ai_cleanup_log (track_id);
