-- =============================================================================
-- MIGRATION 26: HIGH-PERFORMANCE SEARCH & COMPOSITE SPATIAL INDEXES
-- =============================================================================

BEGIN;

-- ۱. ایندکس ترکیبی وضعیت آگهی و تاریخ انتشار (جهت واکشی فوق سریع سورت جدیدترین)
CREATE INDEX IF NOT EXISTS idx_opportunities_published_covering 
ON opportunities (status, published_at DESC) 
WHERE status = 'published';

-- ۲. ایندکس ترکیبی آگهی‌های استخدام فوری
CREATE INDEX IF NOT EXISTS idx_opportunities_urgent_published 
ON opportunities (is_urgent, published_at DESC) 
WHERE status = 'published' AND is_urgent = true;

-- ۳. ایندکس معکوس جدول واسط لوکیشن به آگهی (حیاتی برای JOIN سریع نقشه)
CREATE INDEX IF NOT EXISTS idx_opportunity_locations_reverse 
ON opportunity_locations (location_id, opportunity_id);

-- ۴. ایندکس مهارت‌های آگهی برای جلوگیری از اسکن کامل جدول
CREATE INDEX IF NOT EXISTS idx_opportunity_skills_skill 
ON opportunity_skills (skill_id, opportunity_id);

COMMIT;