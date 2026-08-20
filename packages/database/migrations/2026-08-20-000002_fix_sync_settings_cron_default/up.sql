-- The `cron` crate used to compute next_run expects a 6-field expression
-- (sec min hour day month day-of-week). The initial seed used a 5-field
-- expression, which fails to parse and makes every scheduler tick error out.
-- Fix the seeded default in place for already-migrated databases.

UPDATE sync_settings
SET cron_expression = '0 0 3 * * *'
WHERE cron_expression = '0 3 * * *';
