-- Cached results of the "remote" reference audit (Data Quality > Reference
-- Audit > Remote): re-queries each provider for the current name/title tied
-- to a reference and compares it with what Soundome has stored locally.
-- Populated only when a user explicitly triggers an audit run (never
-- automatically), so results can go stale -- `checked_at` lets the UI show
-- "last checked" and offer to re-run on demand.

CREATE TABLE reference_audit_cache (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    reference_id INTEGER NOT NULL,
    remote_name TEXT,
    similarity_score REAL,
    status TEXT NOT NULL,
    checked_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_reference_audit_cache_reference ON reference_audit_cache (reference_id);
CREATE INDEX idx_reference_audit_cache_entity ON reference_audit_cache (entity_type, entity_id);
