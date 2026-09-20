-- Enable PostGIS Extension
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Ensure standard Spatial Reference System (EPSG:4326) exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM spatial_ref_sys WHERE srid = 4326) THEN
        RAISE EXCEPTION 'SRID 4326 is missing from spatial_ref_sys';
    END IF;
END $$;