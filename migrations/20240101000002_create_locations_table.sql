CREATE TABLE IF NOT EXISTS locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- EPSG:4326 Point Geometry [longitude, latitude]
    coordinates geometry(Point, 4326) NOT NULL,
    address_summary TEXT,
    precision VARCHAR(32) NOT NULL DEFAULT 'exact',
    source VARCHAR(64) NOT NULL DEFAULT 'manual',
    source_id VARCHAR(128),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Spatial GiST Index for 2D Spatial Operations
CREATE INDEX IF NOT EXISTS idx_locations_coordinates_gist 
ON locations USING GIST (coordinates);

-- B-Tree Index for Data Source Provenance Queries
CREATE INDEX IF NOT EXISTS idx_locations_source_source_id 
ON locations (source, source_id);

-- Trigger for auto-updating updated_at
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER set_locations_updated_at
BEFORE UPDATE ON locations
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();