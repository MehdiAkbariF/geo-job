-- =============================================================================
-- MIGRATION: EXTENSIVE CANDIDATE PROFILE & ECOSYSTEM EXPANSION
-- =============================================================================

BEGIN;

-- ۱. ارتقای جدول اصلی کارجویان (مشخصات دموگرافیک، حریم خصوصی مکانی و بخش‌های تکمیلی)
ALTER TABLE candidates
    ADD COLUMN IF NOT EXISTS residence_location_id UUID REFERENCES locations(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS preferred_commute_center_id UUID REFERENCES locations(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS preferred_commute_radius_meters INT DEFAULT 5000,
    ADD COLUMN IF NOT EXISTS is_foreign_national BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS nationality_country_code VARCHAR(3),
    ADD COLUMN IF NOT EXISTS has_disability BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS disability_type VARCHAR(32),
    ADD COLUMN IF NOT EXISTS gender VARCHAR(16),
    ADD COLUMN IF NOT EXISTS military_service_status VARCHAR(32),
    ADD COLUMN IF NOT EXISTS marital_status VARCHAR(16),
    ADD COLUMN IF NOT EXISTS birth_date DATE,
    ADD COLUMN IF NOT EXISTS preferred_category_ids UUID[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS audio_intro_storage_key VARCHAR(255),
    ADD COLUMN IF NOT EXISTS awards JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS certifications JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS academic_projects JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS publications JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS volunteering JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS portfolio_items JSONB NOT NULL DEFAULT '[]'::jsonb;

-- ۲. ارتقای جدول سوابق تحصیلی (اضافه شدن معدل، رده مدرک و سال‌ها)
ALTER TABLE candidate_educations
    ADD COLUMN IF NOT EXISTS degree_level VARCHAR(32) NOT NULL DEFAULT 'bachelor',
    ADD COLUMN IF NOT EXISTS gpa NUMERIC(4, 2),
    ADD COLUMN IF NOT EXISTS start_year INT,
    ADD COLUMN IF NOT EXISTS end_year INT,
    ADD COLUMN IF NOT EXISTS is_current BOOLEAN NOT NULL DEFAULT false;

-- ۳. ارتقای جدول سوابق شغلی (اضافه شدن رده سازمانی، صنعت، شهر، ماه و دستاوردها)
ALTER TABLE candidate_experiences
    ADD COLUMN IF NOT EXISTS activity_field VARCHAR(100),
    ADD COLUMN IF NOT EXISTS seniority_level VARCHAR(64),
    ADD COLUMN IF NOT EXISTS company_industry VARCHAR(100),
    ADD COLUMN IF NOT EXISTS country VARCHAR(64) DEFAULT 'ایران',
    ADD COLUMN IF NOT EXISTS city VARCHAR(64),
    ADD COLUMN IF NOT EXISTS start_month INT,
    ADD COLUMN IF NOT EXISTS start_year INT,
    ADD COLUMN IF NOT EXISTS end_month INT,
    ADD COLUMN IF NOT EXISTS end_year INT,
    ADD COLUMN IF NOT EXISTS achievements TEXT;

-- ۴. ایجاد جدول زبان‌های خارجی (Languages)
CREATE TABLE IF NOT EXISTS candidate_languages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    language_name VARCHAR(64) NOT NULL,
    proficiency_level VARCHAR(32) NOT NULL DEFAULT 'intermediate', -- basic, intermediate, professional_working, fluent, native
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cand_lang_cand ON candidate_languages (candidate_id);

-- ۵. ایجاد جدول معرف‌ها و همکاران سابق (References)
CREATE TABLE IF NOT EXISTS candidate_references (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    full_name VARCHAR(150) NOT NULL,
    organization_name VARCHAR(150) NOT NULL,
    job_title VARCHAR(150) NOT NULL,
    relationship_type VARCHAR(64), -- direct_manager, peer, subordinate, client
    start_year INT,
    end_year INT,
    is_still_colleagues BOOLEAN NOT NULL DEFAULT false,
    phone VARCHAR(32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cand_ref_cand ON candidate_references (candidate_id);

COMMIT;