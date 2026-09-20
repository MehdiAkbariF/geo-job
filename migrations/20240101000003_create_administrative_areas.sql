CREATE TABLE IF NOT EXISTS administrative_areas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES administrative_areas(id) ON DELETE SET NULL,
    country_code VARCHAR(3) NOT NULL, -- ISO-3166-1 alpha-3 (e.g. 'IRN')
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

-- Spatial GiST Index for Point-in-Polygon containment queries
CREATE INDEX IF NOT EXISTS idx_admin_areas_boundary_gist 
ON administrative_areas USING GIST (boundary);

CREATE INDEX IF NOT EXISTS idx_admin_areas_center_gist 
ON administrative_areas USING GIST (center);

CREATE INDEX IF NOT EXISTS idx_admin_areas_hierarchy 
ON administrative_areas (country_code, admin_level, parent_id);

-- Initial Bootstrap Data: Tehran Province, Tehran City, and Sa'adat Abad Neighborhood
DO $$
DECLARE
    v_province_id UUID;
    v_city_id UUID;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM administrative_areas WHERE name = 'استان تهران') THEN
        -- Province: Tehran (admin_level = 4)
        INSERT INTO administrative_areas (id, country_code, admin_level, area_type, name, name_en, boundary, center)
        VALUES (
            gen_random_uuid(),
            'IRN',
            4,
            'province',
            'استان تهران',
            'Tehran Province',
            ST_Multi(ST_GeomFromText('POLYGON((50.5 35.0, 52.5 35.0, 52.5 36.2, 50.5 36.2, 50.5 35.0))', 4326)),
            ST_SetSRID(ST_MakePoint(51.3890, 35.6892), 4326)
        ) RETURNING id INTO v_province_id;

        -- City: Tehran (admin_level = 8)
        INSERT INTO administrative_areas (id, parent_id, country_code, admin_level, area_type, name, name_en, boundary, center)
        VALUES (
            gen_random_uuid(),
            v_province_id,
            'IRN',
            8,
            'city',
            'تهران',
            'Tehran',
            ST_Multi(ST_GeomFromText('POLYGON((51.15 35.55, 51.65 35.55, 51.65 35.85, 51.15 35.85, 51.15 35.55))', 4326)),
            ST_SetSRID(ST_MakePoint(51.3890, 35.6892), 4326)
        ) RETURNING id INTO v_city_id;

        -- Neighborhood: Sa'adat Abad (admin_level = 10)
        INSERT INTO administrative_areas (id, parent_id, country_code, admin_level, area_type, name, name_en, boundary, center)
        VALUES (
            gen_random_uuid(),
            v_city_id,
            'IRN',
            10,
            'neighborhood',
            'سعادت‌آباد',
            'Saadat Abad',
            ST_Multi(ST_GeomFromText('POLYGON((51.36 35.77, 51.39 35.77, 51.39 35.80, 51.36 35.80, 51.36 35.77))', 4326)),
            ST_SetSRID(ST_MakePoint(51.375, 35.785), 4326)
        );
    END IF;
END $$;