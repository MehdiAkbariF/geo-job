-- =============================================================================
-- MIGRATION 27: MAP MARKERS CLUSTERING OPTIMIZATION
-- =============================================================================
-- این migration ایندکس‌های لازم برای endpoint جدید /opportunities/map-markers
-- را اضافه می‌کند که کلاستربندی را به صورت سرور-ساید با ST_SnapToGrid انجام می‌دهد.
-- =============================================================================

BEGIN;

-- ۱. ایندکس ترکیبی برای فیلتر اصلی map markers (status + workplace + category)
CREATE INDEX IF NOT EXISTS idx_opportunities_map_markers
ON opportunities (status, workplace_type, category_id)
WHERE status = 'published';

-- ۲. ایندکس روی (status, is_urgent) برای aggregation سریع urgent_count
CREATE INDEX IF NOT EXISTS idx_opportunities_urgent_aggregate
ON opportunities (status, is_urgent)
WHERE status = 'published';

-- ۳. ایندکس روی opportunity_locations برای JOIN معکوس (location → opportunity)
CREATE INDEX IF NOT EXISTS idx_opportunity_locations_location_opportunity
ON opportunity_locations (location_id, opportunity_id);

-- ۴. ایندکس روی salary_min و salary_max برای MIN/MAX aggregation سریع
CREATE INDEX IF NOT EXISTS idx_opportunities_salary_aggregate
ON opportunities (salary_min, salary_max)
WHERE status = 'published';

-- ۵. ANALYZE برای به‌روزرسانی آمار planner
ANALYZE opportunities;
ANALYZE opportunity_locations;
ANALYZE locations;

COMMIT;