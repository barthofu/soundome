-- Add SOUNDOME_ID anchor column for bidirectional filesystem binding.
-- Nullable to support rows that existed before this migration.
-- Note: SQLite doesn't support ADD COLUMN with UNIQUE constraint on existing tables
-- with data, so we make it nullable and non-unique here. Consider adding a unique
-- constraint in a future migration if all soundome_ids are populated.

ALTER TABLE track ADD COLUMN soundome_id TEXT;
