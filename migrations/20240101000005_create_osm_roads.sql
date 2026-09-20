CREATE TABLE IF NOT EXISTS osm_roads (
    id BIGINT PRIMARY KEY,
    name TEXT,
    highway VARCHAR(32) NOT NULL,
    geom geometry(LineString, 4326) NOT NULL,
    z_order INT NOT NULL DEFAULT 0
);

-- Spatial GiST Index for Fast Vector Tile Clipping
CREATE INDEX IF NOT EXISTS idx_osm_roads_geom_gist 
ON osm_roads USING GIST (geom);

-- B-Tree Index for Zoom-based Filtering
CREATE INDEX IF NOT EXISTS idx_osm_roads_highway 
ON osm_roads (highway);