use sqlx::PgPool;
use uuid::Uuid;
use rand::Rng;

pub async fn clean_seeded_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Cleaning up all heavy test data from database...");

    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM opportunity_skills WHERE opportunity_id IN (SELECT id FROM opportunities WHERE description LIKE '%[HEAVY_SEED]%')").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM opportunity_locations WHERE opportunity_id IN (SELECT id FROM opportunities WHERE description LIKE '%[HEAVY_SEED]%')").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM opportunities WHERE description LIKE '%[HEAVY_SEED]%'").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM company_memberships WHERE company_id IN (SELECT id FROM companies WHERE description LIKE '%[HEAVY_SEED]%')").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM company_locations WHERE company_id IN (SELECT id FROM companies WHERE description LIKE '%[HEAVY_SEED]%')").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM companies WHERE description LIKE '%[HEAVY_SEED]%'").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM locations WHERE source = 'opportunity' AND source_id LIKE 'heavy_seed_%'").execute(&mut *tx).await?;

    tx.commit().await?;
    tracing::info!("Clean up completed successfully!");
    Ok(())
}

pub async fn seed_heavy_tehran_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    clean_seeded_data(pool).await?;

    tracing::info!("Starting HEAVY Tehran Seeding (1500+ jobs across all neighborhoods)...");

    let mut tx = pool.begin().await?;

    // ۱. دسته‌بندی شغلی (استفاده از کوئری ایمن بدون نیاز به ایندکس Unique)
    let cat_id = match sqlx::query_scalar::<_, Uuid>("SELECT id FROM categories WHERE slug = 'it-software'")
        .fetch_optional(&mut *tx)
        .await? 
    {
        Some(id) => id,
        None => {
            let new_id = Uuid::new_v4();
            sqlx::query(r#"
                INSERT INTO categories (id, name, slug, description, created_at, updated_at)
                VALUES ($1, 'فناوری اطلاعات و نرم‌افزار', 'it-software', 'توسعه نرم‌افزار [HEAVY_SEED]', NOW(), NOW())
            "#)
            .bind(new_id)
            .execute(&mut *tx)
            .await?;
            new_id
        }
    };

    // ۲. مراکز محلات تهران برای توزیع طبیعی نقاط
    let tehran_hubs = vec![
        ("ونک", 51.403, 35.757),
        ("سعادت‌آباد", 51.371, 35.788),
        ("تجریش", 51.424, 35.808),
        ("میدان انقلاب", 51.391, 35.700),
        ("پیروزی", 51.455, 35.705),
        ("تهرانپارس", 51.532, 35.733),
        ("صادقیه", 51.345, 35.719),
        ("شهرک غرب", 51.365, 35.760),
        ("جردن", 51.417, 35.759),
        ("فردوسی", 51.419, 35.690),
        ("جنت‌آباد", 51.312, 35.765),
        ("پونک", 51.342, 35.762),
    ];

    // ۳. ساخت ۵۰ شرکت تستی
    let mut company_ids = Vec::new();
    for i in 0..50 {
        let comp_id = Uuid::new_v4();
        sqlx::query(r#"
            INSERT INTO companies (id, name, slug, description, website, verification_status, created_at, updated_at)
            VALUES ($1, $2, $3, 'شرکت پیشرو در فناوری اطلاعات [HEAVY_SEED]', 'https://tech-corp.ir', 'verified', NOW(), NOW())
        "#)
        .bind(comp_id)
        .bind(format!("استارتاپ فناور {} شماره {}", i + 1, i))
        .bind(format!("heavy-tech-corp-{}", i))
        .execute(&mut *tx).await?;

        company_ids.push(comp_id);
    }

    let titles = vec![
        "توسعه‌دهنده ارشد Rust", "برنامه‌نویس React.js", "مدیر محصول", 
        "مهندس DevOps", "کارشناس کنترل کیفیت QA", "تحلیلگر داده Data Analyst",
        "توسعه‌دهنده پایتون", "مدیر فنی CTO", "طراح UI/UX"
    ];

    let workplaces = vec!["onsite", "hybrid", "remote"];
    let exp_levels = vec!["junior", "mid_level", "senior", "lead"];

    let mut rng = rand::thread_rng();
    let total_jobs = 1500;

    tracing::info!("Generating {} opportunities across Tehran...", total_jobs);

    for j in 0..total_jobs {
        let hub = tehran_hubs[rng.gen_range(0..tehran_hubs.len())];
        let lon_offset: f64 = rng.gen_range(-0.035..0.035);
        let lat_offset: f64 = rng.gen_range(-0.035..0.035);
        let lon = hub.1 + lon_offset;
        let lat = hub.2 + lat_offset;

        let loc_id = Uuid::new_v4();
        let source_id = format!("heavy_seed_loc_{}", j);

        // درج لوکیشن در PostGIS
        sqlx::query(r#"
            INSERT INTO locations (id, coordinates, address_summary, precision, source, source_id, metadata, created_at, updated_at)
            VALUES ($1, ST_SetSRID(ST_MakePoint($2, $3), 4326), $4, 'exact', 'opportunity', $5, $6, NOW(), NOW())
        "#)
        .bind(loc_id)
        .bind(lon)
        .bind(lat)
        .bind(format!("تهران، محدوده {} (پلاک تستی {})", hub.0, j))
        .bind(source_id)
        .bind(serde_json::json!({ "neighborhood": hub.0, "heavy_seed": true }))
        .execute(&mut *tx).await?;

        // درج فرصت شغلی
        let opp_id = Uuid::new_v4();
        let title = titles[rng.gen_range(0..titles.len())];
        let comp_id = company_ids[rng.gen_range(0..company_ids.len())];
        let wp = workplaces[rng.gen_range(0..workplaces.len())];
        let exp = exp_levels[rng.gen_range(0..exp_levels.len())];
        let s_min = rng.gen_range(30..90) * 1_000_000;
        let s_max = s_min + 30_000_000;

        sqlx::query(r#"
            INSERT INTO opportunities (
                id, company_id, title, description, category_id,
                opportunity_type, workplace_type, remote_scope, experience_level,
                salary_min, salary_max, salary_currency, salary_period,
                status, published_at, expires_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                'full_time', $6, 'country', $7,
                $8, $9, 'IRR', 'monthly',
                'published', NOW(), NOW() + INTERVAL '60 days', NOW(), NOW()
            )
        "#)
        .bind(opp_id)
        .bind(comp_id)
        .bind(format!("{} (محدوده {})", title, hub.0))
        .bind(format!("فرصت شغلی عالی در منطقه {} تهران. تسلط به مهارت‌های مرتبط الزامی است. [HEAVY_SEED]", hub.0))
        .bind(cat_id)
        .bind(wp)
        .bind(exp)
        .bind(rust_decimal::Decimal::new(s_min, 0))
        .bind(rust_decimal::Decimal::new(s_max, 0))
        .execute(&mut *tx).await?;

        // اتصال آگهی به لوکیشن
        sqlx::query("INSERT INTO opportunity_locations (opportunity_id, location_id) VALUES ($1, $2)")
            .bind(opp_id)
            .bind(loc_id)
            .execute(&mut *tx).await?;

        if (j + 1) % 300 == 0 {
            tracing::info!("Seeded {} / {} jobs...", j + 1, total_jobs);
        }
    }

    tx.commit().await?;
    tracing::info!("HEAVY TEHRAN SEEDING COMPLETED! Successfully ingested 1500 jobs.");
    Ok(())
}