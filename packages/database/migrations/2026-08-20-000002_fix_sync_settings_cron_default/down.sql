UPDATE sync_settings
SET cron_expression = '0 3 * * *'
WHERE cron_expression = '0 0 3 * * *';
