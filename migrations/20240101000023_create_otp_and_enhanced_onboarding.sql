-- =============================================================================
-- MIGRATION: PHONE OTP AUTHENTICATION & MULTI-PERSONA ONBOARDING (PROJECTS)
-- =============================================================================

BEGIN;

-- ۱. جدول کدهای تایید یکبار مصرف پیامکی (OTP Verifications)
CREATE TABLE IF NOT EXISTS phone_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone VARCHAR(20) NOT NULL,
    code_hash VARCHAR(100) NOT NULL,
    attempts INT NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_phone_verifications_phone 
ON phone_verifications (phone, created_at DESC);

-- ۲. ارتقای جدول کاربران (تفکیک ۳ نقش کارجو، کارفرمای حضوری و کارفرمای پروژه‌ای)
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS user_type VARCHAR(32) NOT NULL DEFAULT 'candidate', -- candidate, employer, project_client
    ADD COLUMN IF NOT EXISTS national_id VARCHAR(10),
    ADD COLUMN IF NOT EXISTS is_phone_verified BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_onboarded BOOLEAN NOT NULL DEFAULT false;

-- ۳. ارتقای جدول آگهی‌ها برای پشتیبانی کامل از «درخواست پروژه» و بدون لوکیشن
ALTER TABLE opportunities
    ADD COLUMN IF NOT EXISTS is_project_based BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS project_deadline_days INT,
    ADD COLUMN IF NOT EXISTS project_budget_min NUMERIC(15, 2),
    ADD COLUMN IF NOT EXISTS project_budget_max NUMERIC(15, 2);

CREATE INDEX IF NOT EXISTS idx_users_phone ON users (phone);
CREATE INDEX IF NOT EXISTS idx_opportunities_project ON opportunities (is_project_based) WHERE status = 'published';

COMMIT;