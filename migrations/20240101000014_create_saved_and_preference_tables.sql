-- 1. Saved Opportunities Table (Independent entity - Section 46)
CREATE TABLE IF NOT EXISTS saved_opportunities (
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    opportunity_id UUID NOT NULL REFERENCES opportunities(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (candidate_id, opportunity_id)
);

CREATE INDEX IF NOT EXISTS idx_saved_opps_cand ON saved_opportunities (candidate_id);

-- 2. Saved Companies Table (Independent entity - Section 47)
CREATE TABLE IF NOT EXISTS saved_companies (
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (candidate_id, company_id)
);

CREATE INDEX IF NOT EXISTS idx_saved_comps_cand ON saved_companies (candidate_id);

-- 3. Saved Searches Table (Discovery criteria, NOT transient map state - Section 48)
CREATE TABLE IF NOT EXISTS saved_searches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    title VARCHAR(150) NOT NULL,
    criteria JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_saved_searches_cand ON saved_searches (candidate_id);

-- 4. Candidate Preferences Table (Independent from search - Section 49)
CREATE TABLE IF NOT EXISTS candidate_preferences (
    candidate_id UUID PRIMARY KEY REFERENCES candidates(id) ON DELETE CASCADE,
    preferred_workplace_types VARCHAR(32)[] NOT NULL DEFAULT '{}',
    preferred_opportunity_types VARCHAR(32)[] NOT NULL DEFAULT '{}',
    expected_salary_min NUMERIC(15, 2),
    salary_currency VARCHAR(3) NOT NULL DEFAULT 'IRR',
    remote_only BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger for candidate preferences timestamp
CREATE OR REPLACE TRIGGER set_candidate_preferences_updated_at
BEFORE UPDATE ON candidate_preferences
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();