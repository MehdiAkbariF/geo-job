-- Migration: Job Invitations and Talent Discovery Indexes
BEGIN;

-- ۱. جدول دعوت‌نامه‌های رسمی کارفرما به کارجو
CREATE TABLE IF NOT EXISTS job_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    opportunity_id UUID NOT NULL REFERENCES opportunities(id) ON DELETE CASCADE,
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    sender_user_id UUID NOT NULL REFERENCES users(id),
    message TEXT,
    status VARCHAR(32) NOT NULL DEFAULT 'pending', -- pending, accepted, declined
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_invitation_opp_cand UNIQUE (opportunity_id, candidate_id)
);

CREATE INDEX IF NOT EXISTS idx_invitations_cand ON job_invitations (candidate_id, status);
CREATE INDEX IF NOT EXISTS idx_invitations_company ON job_invitations (company_id);

-- ۲. ایندکس‌های پرسرعت برای رصد استعدادها روی نقشه
CREATE INDEX IF NOT EXISTS idx_candidates_job_status ON candidates (job_search_status) WHERE job_search_status != 'not_looking';
CREATE INDEX IF NOT EXISTS idx_candidates_preferred_city ON candidates (preferred_city);

COMMIT;