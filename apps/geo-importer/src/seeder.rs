use sqlx::PgPool;
use uuid::Uuid;

pub async fn clean_seeded_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Cleaning up all seeded test data from database...");

    let mut tx = pool.begin().await?;

    // پاک‌سازی روابط و موجودیت‌های تستی
    sqlx::query("DELETE FROM opportunity_skills WHERE opportunity_id IN (SELECT id FROM opportunities WHERE description LIKE '%[SEED]%')").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM opportunity_locations WHERE opportunity_id IN (SELECT id FROM opportunities WHERE description LIKE '%[SEED]%')").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM opportunities WHERE description LIKE '%[SEED]%'").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM company_memberships WHERE company_id IN (SELECT id FROM companies WHERE description LIKE '%[SEED]%')").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM company_locations WHERE company_id IN (SELECT id FROM companies WHERE description LIKE '%[SEED]%')").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM companies WHERE description LIKE '%[SEED]%'").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM locations WHERE source = 'opportunity' AND source_id LIKE 'seed_%'").execute(&mut *tx).await?;

    tx.commit().await?;
    tracing::info!("Clean up completed successfully! Database is now pristine.");
    Ok(())
}

pub async fn seed_iran_test_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // ابتدا در صورت وجود دیتای تستی قدیمی، آن را پاک می‌کنیم
    clean_seeded_data(pool).await?;

    tracing::info!("Starting comprehensive Iranian Jobs & Spatial Seeding...");

    let mut tx = pool.begin().await?;

    // ۱. دسته‌بندی‌های شغلی استاندارد (استفاده از کوئری امن بدون نیاز به ایندکس Unique)
    let categories = vec![
        ("فناوری اطلاعات و نرم‌افزار", "it-software", "برنامه‌نویسی، دواپس، شبکه و زیرساخت"),
        ("بازاریابی و فروش", "marketing-sales", "دیجیتال مارکتینگ، سئو و فروش سازمانی"),
        ("مالی و حسابداری", "finance-accounting", "حسابرسی، حسابداری و امور مالیاتی"),
        ("طراحی و محصول", "design-product", "طراحی رابط کاربری، گرافیک و مدیریت محصول"),
    ];

    let mut cat_ids = Vec::new();
    for (name, slug, desc) in categories {
        let existing_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM categories WHERE slug = $1")
            .bind(slug)
            .fetch_optional(&mut *tx)
            .await?;

        let id = match existing_id {
            Some(id) => id,
            None => {
                let new_id = Uuid::new_v4();
                sqlx::query("INSERT INTO categories (id, name, slug, description, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())")
                    .bind(new_id)
                    .bind(name)
                    .bind(slug)
                    .bind(desc)
                    .execute(&mut *tx).await?;
                new_id
            }
        };
        cat_ids.push(id);
    }

    let cat_it = cat_ids[0];

    // ۲. مهارت‌های استاندارد بازار کار
    let skills = vec![
        "Rust", "React", "PostgreSQL", "Docker", "سئو و تولید محتوا",
        "حسابداری سپیدار", "UI/UX Design", "مدیریت محصول"
    ];

    for s_name in skills {
        let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM skills WHERE name = $1")
            .bind(s_name)
            .fetch_optional(&mut *tx)
            .await?;

        if existing.is_none() {
            sqlx::query("INSERT INTO skills (id, name, created_at) VALUES ($1, $2, NOW())")
                .bind(Uuid::new_v4())
                .bind(s_name)
                .execute(&mut *tx).await?;
        }
    }

    // ۳. نقاط مکانی دقیق در شهرهای ایران (تهران، اصفهان، مشهد، شیراز، تبریز)
    let locations_data = vec![
        // تهران
        ("ونک - جردن", 51.4172, 35.7592, "تهران، خیابان نلسون ماندلا، برج نگین"),
        ("سعادت‌آباد", 51.3685, 35.7832, "تهران، سعادت‌آباد، میدان کاج، خیابان مروارید"),
        ("میدان انقلاب", 51.3912, 35.7008, "تهران، خیابان انقلاب، تقاطع ۱۶ آذر، ساختمان فناوری"),
        ("میدان آزادی", 51.3364, 35.6997, "تهران، بزرگراه جناح، نبش خیابان دانش"),
        ("عباس‌آباد - بهشتی", 51.4285, 35.7315, "تهران، خیابان شهید بهشتی، پلاک ۲۱۰"),
        ("پارک فناوری پردیس", 51.7820, 35.7350, "تهران، کیلومتر ۲۰ جاده دماوند، پارک فناوری پردیس"),
        // اصفهان
        ("اصفهان - چهارباغ عباسی", 51.6660, 32.6546, "اصفهان، خیابان چهارباغ عباسی، کوچه سینما"),
        ("اصفهان - شهرک علمی و تحقیقاتی", 51.5240, 32.7190, "اصفهان، دانشگاه صنعتی اصفهان، شهرک فناوری"),
        // مشهد
        ("مشهد - بلوار سجاد", 59.5400, 36.3150, "مشهد، بلوار سجاد، تقاطع خیام، برج اداری باران"),
        ("مشهد - احمدآباد", 59.5700, 36.3000, "مشهد، خیابان احمدآباد، نبش خیابان ملاصدرا"),
        // شیراز
        ("شیراز - بلوار چمران", 52.5200, 29.6350, "شیراز، بلوار شهید چمران، روبروی بیمارستان اردیبهشت"),
        // تبریز
        ("تبریز - ایل‌گلی", 46.3600, 38.0500, "تبریز، بلوار ایل‌گلی، برج فناوری اطلاعات"),
    ];

    let mut loc_ids = Vec::new();
    for (i, (title, lon, lat, address)) in locations_data.iter().enumerate() {
        let loc_id = Uuid::new_v4();
        sqlx::query(r#"
            INSERT INTO locations (id, coordinates, address_summary, precision, source, source_id, metadata, created_at, updated_at)
            VALUES ($1, ST_SetSRID(ST_MakePoint($2, $3), 4326), $4, 'exact', 'opportunity', $5, $6, NOW(), NOW())
        "#)
        .bind(loc_id)
        .bind(lon)
        .bind(lat)
        .bind(address)
        .bind(format!("seed_loc_{}", i))
        .bind(serde_json::json!({ "area": title, "seed": true }))
        .execute(&mut *tx).await?;

        loc_ids.push((loc_id, *title, *lon, *lat));
    }

    // ۴. ثبت شرکت‌ها
    let companies_data = vec![
        ("ابر دیوان", "divan-cloud", "ارائه دهنده راهکارهای نوین پردازش ابری [SEED]", "https://divan.ir"),
        ("نوآوران تارگاه", "targah-tech", "سامانه پرداخت آنلاین و راهکارهای تجارت الکترونیک [SEED]", "https://targah.ir"),
        ("پرتو هوش پارس", "parto-ai", "هوش مصنوعی و بینایی ماشین صنعتی [SEED]", "https://parto.ai"),
        ("ارتباطات سپهر", "sepehr-telecom", "شبکه، دیتاسنتر و زیرساخت مخابراتی [SEED]", "https://sepehr.ir"),
        ("زاگرس گستران اصفهان", "zagros-isfahan", "توسعه نرم‌افزارهای سازمانی و زنجیره تامین [SEED]", "https://zagros.co"),
        ("داده‌پردازی توس مشهد", "toos-data", "طراحی سیستم‌های یکپارچه شهری و رزرواسیون [SEED]", "https://toosdata.ir"),
    ];

    let mut company_ids = Vec::new();
    for (c_name, slug, desc, web) in companies_data {
        let comp_id = Uuid::new_v4();
        sqlx::query(r#"
            INSERT INTO companies (id, name, slug, description, website, verification_status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'verified', NOW(), NOW())
        "#)
        .bind(comp_id)
        .bind(c_name)
        .bind(slug)
        .bind(desc)
        .bind(web)
        .execute(&mut *tx).await?;

        company_ids.push(comp_id);
    }

    // ۵. ثبت فرصت‌های شغلی با حقوق‌های واقعی و اتصال به لوکیشن‌های PostGIS
    let job_titles = vec![
        ("توسعه‌دهنده ارشد Rust و سیستم‌های توزیع‌شده", "onsite", "full_time", "senior", 65_000_000, 95_000_000),
        ("برنامه‌نویس React و Next.js", "hybrid", "full_time", "mid_level", 35_000_000, 50_000_000),
        ("مدیر محصول (Product Manager)", "onsite", "full_time", "lead", 55_000_000, 80_000_000),
        ("کارشناس سئو و بهینه‌سازی وب", "remote", "contract", "junior", 22_000_000, 30_000_000),
        ("مهندس دواپس (DevOps & Docker)", "hybrid", "full_time", "senior", 60_000_000, 85_000_000),
        ("حسابدار و کارشناس مالیاتی", "onsite", "full_time", "mid_level", 25_000_000, 38_000_000),
    ];

    let mut opp_count = 0;
    for (loc_idx, (loc_id, loc_name, _, _)) in loc_ids.iter().enumerate() {
        for job_idx in 0..2 {
            let (title, workplace, opp_type, exp, s_min, s_max) = job_titles[(loc_idx + job_idx) % job_titles.len()];
            let comp_id = company_ids[loc_idx % company_ids.len()];
            let opp_id = Uuid::new_v4();

            sqlx::query(r#"
                INSERT INTO opportunities (
                    id, company_id, title, description, category_id,
                    opportunity_type, workplace_type, remote_scope, experience_level,
                    salary_min, salary_max, salary_currency, salary_period,
                    status, published_at, expires_at, created_at, updated_at
                )
                VALUES (
                    $1, $2, $3, $4, $5,
                    $6, $7, 'country', $8,
                    $9, $10, 'IRR', 'monthly',
                    'published', NOW(), NOW() + INTERVAL '30 days', NOW(), NOW()
                )
            "#)
            .bind(opp_id)
            .bind(comp_id)
            .bind(format!("{} ({})", title, loc_name))
            .bind(format!("موقعیت شغلی در شرکت مطرح، واقع در محدوده {}. این یک رکورد تستی است. [SEED]", loc_name))
            .bind(cat_it)
            .bind(opp_type)
            .bind(workplace)
            .bind(exp)
            .bind(rust_decimal::Decimal::new(s_min, 0))
            .bind(rust_decimal::Decimal::new(s_max, 0))
            .execute(&mut *tx).await?;

            sqlx::query("INSERT INTO opportunity_locations (opportunity_id, location_id) VALUES ($1, $2)")
                .bind(opp_id)
                .bind(loc_id)
                .execute(&mut *tx).await?;

            opp_count += 1;
        }
    }

    tx.commit().await?;

    tracing::info!("Successfully seeded {} opportunities across Iranian cities!", opp_count);
    Ok(())
}