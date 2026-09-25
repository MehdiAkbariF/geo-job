-- =============================================================================
-- MIGRATION 25: FIX PHONE INDEX & SEED DEMO USERS AND PROFILES (SAFE & IDEMPOTENT)
-- =============================================================================

BEGIN;

-- ۱. رفع تداخل ایندکس شماره موبایل
DROP INDEX IF EXISTS idx_users_phone;
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_phone_unique ON users (phone) WHERE phone IS NOT NULL;

-- ۲. درج ایمن کاربران دموی سیستم (در صورتی که قبلاً وجود نداشته باشند)
INSERT INTO users (id, email, phone, password_hash, status, user_type, is_phone_verified, is_onboarded)
SELECT '11111111-1111-1111-1111-111111111111', 'demo@geojob.ir', '09121111111', '$argon2id$v=19$m=19456,t=2,p=1$placeholder$placeholder', 'active', 'candidate', true, true
WHERE NOT EXISTS (SELECT 1 FROM users WHERE LOWER(email) = 'demo@geojob.ir');

INSERT INTO users (id, email, phone, password_hash, status, user_type, is_phone_verified, is_onboarded)
SELECT '22222222-2222-2222-2222-222222222222', 'employer@geojob.ir', '09122222222', '$argon2id$v=19$m=19456,t=2,p=1$placeholder$placeholder', 'active', 'employer', true, true
WHERE NOT EXISTS (SELECT 1 FROM users WHERE LOWER(email) = 'employer@geojob.ir');

INSERT INTO users (id, email, phone, password_hash, status, user_type, is_phone_verified, is_onboarded)
SELECT '33333333-3333-3333-3333-333333333333', 'verified_employer@geojob.ir', '09123333333', '$argon2id$v=19$m=19456,t=2,p=1$placeholder$placeholder', 'active', 'employer', true, true
WHERE NOT EXISTS (SELECT 1 FROM users WHERE LOWER(email) = 'verified_employer@geojob.ir');

INSERT INTO users (id, email, phone, password_hash, status, user_type, is_phone_verified, is_onboarded)
SELECT '44444444-4444-4444-4444-444444444444', 'pending_employer@geojob.ir', '09124444444', '$argon2id$v=19$m=19456,t=2,p=1$placeholder$placeholder', 'active', 'employer', true, true
WHERE NOT EXISTS (SELECT 1 FROM users WHERE LOWER(email) = 'pending_employer@geojob.ir');

-- ۳. ایجاد پروفایل کارجوی دمو برای هر کاربری که ایمیل demo@geojob.ir دارد
INSERT INTO candidates (id, user_id, first_name, last_name, headline, bio, preferred_city, job_search_status)
SELECT 
    gen_random_uuid(),
    u.id,
    'مهدی',
    'اکبری',
    'توسعه‌دهنده ارشد نرم‌افزار و سیستم‌های مکانی',
    'علاقه‌مند به معماری میکروسرویس و Rust',
    'تهران',
    'actively_looking'
FROM users u
WHERE LOWER(u.email) = 'demo@geojob.ir'
  AND NOT EXISTS (SELECT 1 FROM candidates c WHERE c.user_id = u.id);

-- ۴. ایجاد شرکت دموی کارفرما
INSERT INTO companies (id, name, slug, description, website, business_type, verification_status)
SELECT
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
    'فناوران داده مکان‌محور (دمو)',
    'geo-tech-demo',
    'شرکت ارائه‌دهنده راهکارهای نوین GIS و استخدام هوشمند',
    'https://geojob.ir',
    'corporate',
    'verified'
WHERE NOT EXISTS (SELECT 1 FROM companies WHERE LOWER(slug) = 'geo-tech-demo');

-- ۵. پیوند کارفرمای دمو به شرکت دمو
INSERT INTO company_memberships (company_id, user_id, role)
SELECT
    c.id,
    u.id,
    'owner'
FROM companies c, users u
WHERE LOWER(c.slug) = 'geo-tech-demo'
  AND LOWER(u.email) = 'employer@geojob.ir'
  AND NOT EXISTS (
      SELECT 1 FROM company_memberships cm 
      WHERE cm.company_id = c.id AND cm.user_id = u.id
  );

COMMIT;