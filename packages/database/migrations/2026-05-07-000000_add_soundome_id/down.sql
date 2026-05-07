-- SQLite does not support DROP COLUMN in older versions, but Diesel handles this via recreation.
-- The reverse migration is a no-op; the column will remain until the table is recreated.
SELECT 1;
