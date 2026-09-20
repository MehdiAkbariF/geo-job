CREATE TABLE IF NOT EXISTS opportunities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    category_id UUID NOT NULL REFERENCES categories(id),
    occupation_id UUID REFERENCES occupations(id),
    opportunity_type VARCHAR(32) NOT NULL,
    workplace_type VARCHAR(32) NOT NULL,
    remote_scope VARCHAR(32),
    experience_level VARCHAR(32) NOT NULL,
    salary_min NUMERIC(15, 2), -- Stored with NUMERIC (Section 8)
    salary_max NUMERIC(15, 2),
    salary_currency VARCHAR(3) NOT NULL DEFAULT 'IRR',
    salary_period VARCHAR(16) NOT NULL DEFAULT 'monthly', -- monthly, hourly, yearly
    status VARCHAR(32) NOT NULL DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Enforce exact frozen Opportunity lifecycle states (Section 15)
    CONSTRAINT chk_opportunity_status 
        CHECK (status IN ('draft', 'published', 'paused', 'closed')),

    -- Enforce exact frozen Opportunity types (Section 14)
    CONSTRAINT chk_opportunity_type 
        CHECK (opportunity_type IN ('full_time', 'part_time', 'contract', 'internship', 'freelance', 'temporary')),

    -- Enforce Workplace types (Section 17)
    CONSTRAINT chk_workplace_type 
        CHECK (workplace_type IN ('onsite', 'hybrid', 'remote')),

    -- Enforce Remote scope (Section 18)
    CONSTRAINT chk_remote_scope 
        CHECK (remote_scope IS NULL OR remote_scope IN ('global', 'country', 'region', 'timezone'))
);

CREATE INDEX IF NOT EXISTS idx_opportunities_company ON opportunities (company_id);
CREATE INDEX IF NOT EXISTS idx_opportunities_status_expires ON opportunities (status, expires_at);
CREATE INDEX IF NOT EXISTS idx_opportunities_category ON opportunities (category_id);

-- Opportunity Locations Table (One Opportunity may have multiple locations - Section 17)
CREATE TABLE IF NOT EXISTS opportunity_locations (
    opportunity_id UUID NOT NULL REFERENCES opportunities(id) ON DELETE CASCADE,
    location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    PRIMARY KEY (opportunity_id, location_id)
);

-- Opportunity Skills Table (Canonical skills associated with an Opportunity)
CREATE TABLE IF NOT EXISTS opportunity_skills (
    opportunity_id UUID NOT NULL REFERENCES opportunities(id) ON DELETE CASCADE,
    skill_id UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    is_required BOOLEAN NOT NULL DEFAULT true,
    PRIMARY KEY (opportunity_id, skill_id)
);

-- Trigger for auto-updating updated_at
CREATE OR REPLACE TRIGGER set_opportunities_updated_at
BEFORE UPDATE ON opportunities
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();