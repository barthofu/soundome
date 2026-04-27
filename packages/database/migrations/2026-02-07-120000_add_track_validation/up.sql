-- Add validation fields to track for later manual review

ALTER TABLE track ADD COLUMN needs_validation BOOLEAN NOT NULL CHECK (needs_validation IN (0, 1)) DEFAULT 0;
ALTER TABLE track ADD COLUMN validation_reason TEXT;
