-- =============================================================================
-- MIGRATION: ALLOW PASSWORDLESS & EMAIL-LESS PHONE OTP AUTHENTICATION
-- =============================================================================

BEGIN;

-- مجاز کردن مقدار خالی (NULL) برای ایمیل و پسورد جهت ورود سریع با پیامک
ALTER TABLE users ALTER COLUMN email DROP NOT NULL;
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

COMMIT;