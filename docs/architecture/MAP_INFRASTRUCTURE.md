# Map Infrastructure & Geospatial Platform Architecture

## ۱. فلسفه و اصول معماری
- **PostgreSQL + PostGIS به عنوان منبع یکتای حقیقت (Single Source of Truth):** هیچ عملیات فیلترینگ یا پردازش سنگین هندسی وارد حافظه سرور برنامه نشده و تماماً با ایندکس‌های فضایی `GiST` در دیتابیس اجرا می‌شود.
- **توسعه با رویکرد Earned Complexity:** عدم استفاده از سرویس‌های پیچیده و زودهنگام (بدون Kafka، Elasticsearch یا میکرو‌سرویس‌های توزیع‌شده غیرضروری).
- **استقلال کامل از دامنه کسب‌وکار (Domain Agnostic):** هسته مکانی اطلاعات فرصت‌های شغلی (`Opportunity`) یا بیزینس را نمی‌شناسد؛ بلکه فقط موجودیت مستقل `Location`، لایه نمایشی `MapFeature`، و بخش‌های اداری `AdministrativeArea` را مدیریت می‌کند.

## ۲. قراردادهای استاندارد مکانی (Spatial Conventions)
- **سیستم مختصات:** `WGS84 / EPSG:4326`
- **ترتیب مختصات (RFC 7946):** `[longitude, latitude]`
- **واحد مسافت و شعاع:** متر (`meters`) با محاسبات کروی ژئودتیک PostGIS (`geography`)
- **ترتیب مرز محدوده مستطیلی:** `[west, south, east, north]`
- **فرمت تبادل داده لایه نقشه:** استاندارد GeoJSON (`FeatureCollection`) و تایل‌های باینری وکتور (`MVT / Protobuf`)

## ۳. اجزای Crateهای Workspace
| نام کریت | مسئولیت |
| :--- | :--- |
| `geo-types` | انواع داده اولیه مکانی (`GeoPoint`، `BoundingBox`، `Radius`، `Distance`) و اعتبارسنجی مختصات |
| `geo-domain` | مدل‌های دامنه مکانی مستقل از پایگاه داده (`Location`، `LocationPrecision`) |
| `geo-storage` | لایه اتصال به دیتابیس، استقرار مایگریشن‌های PostGIS و ریپازیتوری مکانی |
| `geo-query` | موتور اجرای کوئری‌های مکانی (BBox، شعاع ژئودتیک، KNN ایندکس‌محور، و خوشه‌بندی داینامیک) |
| `geo-presentation` | مدل‌های لایه نمایش نقشه و تبدیل به استاندارد GeoJSON Feature و Marker DTO |
| `geo-tiles` | سیستم مختصات تایل زوم (Web Mercator) و تولید باینری Vector Tile با تابع بومی `ST_AsMVT` |
| `geo-geocoding` | ژئوگرافی سلسله‌مراتبی اداری و موتور Reverse Geocoding مبتنی بر درون‌یابی نقطه در چندضلعی |
| `map-api` | سرویس HTTP با فریم‌ورک Axum برای ارائه داده به کلاینت‌های وب و موبایل |
| `geo-importer` | ابزار خط‌فرمان جهت پایپ‌لاین ورود استریم داده‌های نقشه و OSM PBF |