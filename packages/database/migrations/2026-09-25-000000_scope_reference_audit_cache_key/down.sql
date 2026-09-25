-- The previous global unique index could only represent one entity's result
-- for a numeric reference id. Keep the oldest such result before restoring it.
DELETE FROM reference_audit_cache
 WHERE id NOT IN (
    SELECT MIN(id)
      FROM reference_audit_cache
     GROUP BY reference_id
 );

DROP INDEX idx_reference_audit_cache_reference;
CREATE UNIQUE INDEX idx_reference_audit_cache_reference
    ON reference_audit_cache (reference_id);
