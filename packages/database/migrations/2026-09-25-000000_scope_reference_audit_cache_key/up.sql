-- Artist, album, and track reference tables each have their own row-id
-- sequence. Therefore `reference_id` alone is not globally unique.
DROP INDEX idx_reference_audit_cache_reference;
CREATE UNIQUE INDEX idx_reference_audit_cache_reference
    ON reference_audit_cache (entity_type, reference_id);
