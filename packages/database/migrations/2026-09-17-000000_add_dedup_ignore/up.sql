-- Manual "not a duplicate" list for the Data Quality > Duplicates review queue.
-- Each row means the pair (id_a, id_b) for a given entity_type has been marked
-- as "not a duplicate" and must be excluded from future duplicate suggestions.
-- id_a is always stored < id_b so a pair can only ever be recorded once,
-- regardless of the order in which it was reviewed.

CREATE TABLE dedup_ignore (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    id_a INTEGER NOT NULL,
    id_b INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_dedup_ignore_pair ON dedup_ignore (entity_type, id_a, id_b);
