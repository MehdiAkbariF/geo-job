-- 1. Platform Admin Users (Section 46, 62 & 109: Separate from CompanyMembership)
CREATE TABLE IF NOT EXISTS admin_users (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(32) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_admin_role CHECK (role IN ('admin', 'moderator', 'support'))
);

-- 2. Company Verifications Table (Section 43 & 55)
CREATE TABLE IF NOT EXISTS company_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    status VARCHAR(32) NOT NULL DEFAULT 'not_started',
    evidence_storage_keys TEXT[] NOT NULL DEFAULT '{}', -- S3 storage keys for private evidence
    reviewer_id UUID REFERENCES users(id),
    review_notes TEXT,
    submitted_at TIMESTAMPTZ,
    reviewed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_verification_status 
        CHECK (status IN ('not_started', 'pending', 'under_review', 'verified', 'rejected', 'expired'))
);

CREATE INDEX IF NOT EXISTS idx_verifications_company ON company_verifications (company_id);

-- 3. Reports Table (Allegation - Section 44 & 58)
CREATE TABLE IF NOT EXISTS reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    opportunity_id UUID NOT NULL REFERENCES opportunities(id) ON DELETE CASCADE,
    reason VARCHAR(100) NOT NULL,
    details TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reports_opp ON reports (opportunity_id);

-- 4. Moderation Cases Table (Actual Review Process - Section 44 & 59)
CREATE TABLE IF NOT EXISTS moderation_cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    opportunity_id UUID NOT NULL REFERENCES opportunities(id) ON DELETE CASCADE,
    status VARCHAR(32) NOT NULL DEFAULT 'open', -- open, in_review, resolved, dismissed
    assigned_moderator_id UUID REFERENCES users(id),
    action_taken VARCHAR(64) NOT NULL DEFAULT 'none', -- none, suspended, approved
    internal_notes TEXT, -- Staff-only private notes (Section 48 & 169)
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_moderation_status CHECK (status IN ('open', 'in_review', 'resolved', 'dismissed'))
);

CREATE INDEX IF NOT EXISTS idx_mod_cases_opp ON moderation_cases (opportunity_id);

-- 5. Audit Log Table (Immutable & Append-Only - Section 47, 63 & 114)
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(64) NOT NULL,
    resource_id UUID NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs (resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs (actor_id);