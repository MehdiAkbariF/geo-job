CREATE TABLE IF NOT EXISTS administrative_areas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES administrative_areas(id) ON DELETE SET NULL,
    country_code VARCHAR(3) NOT NULL, -- ISO-3166-1 alpha-3
    admin_level INT NOT NULL,         -- 2: Country, 4: Province, 6: County, 8: City, 10: Neighborhood
    area_type VARCHAR(32) NOT NULL,   -- 'country', 'province', 'county', 'district', 'city', 'neighborhood'
    name VARCHAR(255) NOT NULL,
    name_en VARCHAR(255),
    boundary geometry(MultiPolygon, 4326) NOT NULL,
    center geometry(Point, 4326),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_admin_areas_boundary_gist 
ON administrative_areas USING GIST (boundary);

CREATE INDEX IF NOT EXISTS idx_admin_areas_center_gist 
ON administrative_areas USING GIST (center);

CREATE INDEX IF NOT EXISTS idx_admin_areas_hierarchy 
ON administrative_areas (country_code, admin_level, parent_id);