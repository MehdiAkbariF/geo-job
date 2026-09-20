CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    opportunity_id UUID NOT NULL REFERENCES opportunities(id) ON DELETE CASCADE,
    resume_id UUID REFERENCES candidate_resumes(id) ON DELETE SET NULL,
    cover_letter TEXT,
    status VARCHAR(32) NOT NULL DEFAULT 'submitted',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Enforce exact frozen Application lifecycle states (Section 20)
    CONSTRAINT chk_application_status 
        CHECK (status IN ('submitted', 'reviewing', 'interview', 'accepted', 'rejected')),

    -- Duplicate Prevention Rule: Candidate MUST NOT submit duplicate applications (Section 22)
    CONSTRAINT uq_candidate_opportunity 
        UNIQUE (candidate_id, opportunity_id)
);

CREATE INDEX IF NOT EXISTS idx_applications_candidate ON applications (candidate_id);
CREATE INDEX IF NOT EXISTS idx_applications_opportunity ON applications (opportunity_id);

-- Trigger for auto-updating updated_at
CREATE OR REPLACE TRIGGER set_applications_updated_at
BEFORE UPDATE ON applications
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();