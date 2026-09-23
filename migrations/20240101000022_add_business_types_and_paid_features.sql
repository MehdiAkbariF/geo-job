-- =============================================================================
-- MIGRATION: BUSINESS TYPES, LOCAL SHOP ATTRIBUTES & PAID PROMOTIONS
-- =============================================================================

BEGIN;

-- ۱. افزودن نوع کسب‌وکار و شماره پروانه صنفی به شرکت‌ها / کارفرمایان
ALTER TABLE companies
    ADD COLUMN IF NOT EXISTS business_type VARCHAR(32) NOT NULL DEFAULT 'corporate', -- corporate, retail_shop, restaurant_cafe, clinic_office, workshop
    ADD COLUMN IF NOT EXISTS trade_license_number VARCHAR(64);

-- ۲. افزودن فیلدهای محلی اصناف و قابلیت‌های ارتقای پولی به آگهی‌ها
ALTER TABLE opportunities
    ADD COLUMN IF NOT EXISTS is_urgent BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_featured BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS working_hours VARCHAR(64),
    ADD COLUMN IF NOT EXISTS gender_preference VARCHAR(16) NOT NULL DEFAULT 'any', -- any, female, male
    ADD COLUMN IF NOT EXISTS has_insurance BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS laddered_at TIMESTAMPTZ;

-- ایندکس‌های بهینه‌سازی نقشه و فیلترها
CREATE INDEX IF NOT EXISTS idx_opportunities_featured_urgent 
ON opportunities (is_featured, is_urgent, status) 
WHERE status = 'published';

CREATE INDEX IF NOT EXISTS idx_companies_business_type 
ON companies (business_type);

COMMIT;