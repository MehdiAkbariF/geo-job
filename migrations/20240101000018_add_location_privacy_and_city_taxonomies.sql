-- Migration: Add privacy toggle for candidate exact location and seed standard city boundaries
BEGIN;

ALTER TABLE candidates
    ADD COLUMN IF NOT EXISTS show_exact_location_to_employers BOOLEAN NOT NULL DEFAULT false;

COMMIT;