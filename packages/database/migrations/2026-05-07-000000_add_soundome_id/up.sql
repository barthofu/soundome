-- Add SOUNDOME_ID anchor column for bidirectional filesystem binding.
-- Nullable to support rows that existed before this migration.

ALTER TABLE track ADD COLUMN soundome_id TEXT UNIQUE;
