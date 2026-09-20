-- Formal table-level UNIQUE constraint for idempotent batch inserts and ON CONFLICT inference
ALTER TABLE locations 
ADD CONSTRAINT uq_locations_source_source_id UNIQUE (source, source_id);