-- =============================================================================
-- MIGRATION: SPATIAL JOB RADARS (GEOFENCING) & IN-APP NOTIFICATIONS
-- =============================================================================

BEGIN;

-- ۱. ارتقای جدول رادارها (saved_searches) به ژئوفنس واقعی مکانی PostGIS
ALTER TABLE saved_searches
    ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS center_point GEOGRAPHY(Point, 4326),
    ADD COLUMN IF NOT EXISTS radius_meters INT NOT NULL DEFAULT 3000,
    ADD COLUMN IF NOT EXISTS keywords VARCHAR(150),
    ADD COLUMN IF NOT EXISTS min_salary NUMERIC(15, 2),
    ADD COLUMN IF NOT EXISTS workplace_type VARCHAR(32),
    ADD COLUMN IF NOT EXISTS category_id UUID REFERENCES categories(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS notify_in_app BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS notify_sms BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS last_triggered_at TIMESTAMPTZ;

-- برداشتن قید not null از candidate_id برای پشتیبانی همزمان از user_id و candidate_id
ALTER TABLE saved_searches ALTER COLUMN candidate_id DROP NOT NULL;

-- مقداردهی user_id برای رکوردهای قبلی
UPDATE saved_searches s
SET user_id = c.user_id
FROM candidates c
WHERE s.candidate_id = c.id AND s.user_id IS NULL;

-- ایندکس فضایی فوق سریع GIST برای تطبیق معکوس رادارها (Reverse Geofence Index)
CREATE INDEX IF NOT EXISTS idx_saved_searches_spatial 
ON saved_searches USING GIST(center_point) 
WHERE is_active = true AND center_point IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_saved_searches_user_active 
ON saved_searches (user_id, is_active);


-- ۲. ایجاد جدول اعلان‌های درون‌برنامه‌ای (In-App Notification Center)
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    message TEXT NOT NULL,
    action_url VARCHAR(255),
    notification_type VARCHAR(50) NOT NULL DEFAULT 'radar_match',
    is_read BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notifications_user_unread 
ON notifications (user_id, is_read, created_at DESC);

COMMIT;