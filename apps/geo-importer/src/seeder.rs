use sqlx::PgPool;
use uuid::Uuid;
use rand::Rng;
use rust_decimal::Decimal;

pub async fn clean_seeded_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Cleaning up all test data from database...");

    let mut tx = pool.begin().await?;

    sqlx::query(r#"
        TRUNCATE TABLE 
            applications,
            saved_searches,
            saved_opportunities,
            saved_companies,
            candidate_preferences,
            candidate_resumes,
            candidate_educations,
            candidate_experiences,
            candidate_skills,
            candidates,
            opportunity_skills,
            opportunity_locations,
            opportunities,
            company_locations,
            company_memberships,
            companies,
            locations
        RESTART IDENTITY CASCADE;
    "#)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!("Clean up completed successfully!");
    Ok(())
}

pub async fn seed_10k_national_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    clean_seeded_data(pool).await?;

    tracing::info!("Starting 10,000 High-Volume Opportunities Generation across Iranian Cities...");

    let mut tx = pool.begin().await?;

    // ۱. درج یا بازاستفاده ایمن از دسته‌بندی‌های استاندارد
    let categories_data = [
        ("فناوری اطلاعات و نرم‌افزار", "software-it"),
        ("مدیریت محصول و طراحی UI/UX", "product-design"),
        ("دواپس و زیرساخت ابری", "devops-cloud"),
        ("مالی و حسابداری", "finance-accounting"),
        ("دیجیتال مارکتینگ و سئو", "digital-marketing"),
        ("هوش مصنوعی و علم داده", "ai-data"),
    ];

    let mut cat_ids = Vec::new();
    for (name, slug) in categories_data {
        let existing_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM categories WHERE LOWER(slug) = LOWER($1)"
        )
        .bind(slug)
        .fetch_optional(&mut *tx)
        .await?;

        let cid = match existing_id {
            Some(id) => id,
            None => {
                sqlx::query_scalar(
                    "INSERT INTO categories (name, slug, description) VALUES ($1, $2, $3) RETURNING id"
                )
                .bind(name)
                .bind(slug)
                .bind(format!("دسته‌بندی {}", name))
                .fetch_one(&mut *tx)
                .await?
            }
        };
        cat_ids.push(cid);
    }

    // ۲. درج یا بازاستفاده ایمن از مهارت‌ها
    let skills_data = [
        "Rust", "React", "Next.js", "TypeScript", "PostgreSQL",
        "Docker", "Kubernetes", "Figma", "SEO", "Python", "Go", "حسابداری"
    ];
    let mut skill_ids = Vec::new();
    for sname in skills_data {
        let existing_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM skills WHERE LOWER(name) = LOWER($1)"
        )
        .bind(sname)
        .fetch_optional(&mut *tx)
        .await?;

        let sid = match existing_id {
            Some(id) => id,
            None => {
                sqlx::query_scalar(
                    "INSERT INTO skills (name) VALUES ($1) RETURNING id"
                )
                .bind(sname)
                .fetch_one(&mut *tx)
                .await?
            }
        };
        skill_ids.push(sid);
    }

    // ۳. ایجاد ۵۰ شرکت در سراسر کشور با اسلاگ یکتا
    let mut company_ids = Vec::new();
    let company_names = [
        "ابر دیوان", "نوآوران تارگاه", "پرتو هوش پارس", "زاگرس گستران اصفهان",
        "داده‌پردازی توس مشهد", "ارتباطات سپهر شیراز", "آران‌تک تبریز", "کاسپین داده رشت",
        "البرز سامانه کرج", "فناوری اطلاعات پایتخت", "رایانش ابری زاگرس", "هوش مصنوعی خاورمیانه",
        "داده‌گستر کاسپین", "فناوران نوین توس", "پارس پلتفرم", "ارتباط نوین شیراز"
    ];

    for i in 0..60 {
        let comp_id = Uuid::new_v4();
        let base_name = company_names[i % company_names.len()];
        let name = format!("{} (شعبه {})", base_name, (i / company_names.len()) + 1);
        let random_suffix = &Uuid::new_v4().to_string()[..6];
        let slug = format!("company-{}-{}", i + 1, random_suffix);

        sqlx::query(r#"
            INSERT INTO companies (id, name, slug, description, website, verification_status, created_at, updated_at)
            VALUES ($1, $2, $3, 'شرکت فناوری ثبت‌شده در سامانه [BULK_SEED]', 'https://geojob.ir', 'verified', NOW(), NOW())
        "#)
        .bind(comp_id)
        .bind(name)
        .bind(slug)
        .execute(&mut *tx).await?;

        company_ids.push(comp_id);
    }

    // ۴. قطب‌های شهری برای توزیع دقیق ۱۰,۰۰۰ لوکیشن در محله‌های واقعی
    struct CityHub {
        city_name: &'static str,
        hubs: Vec<(&'static str, f64, f64)>,
        job_target: usize,
    }

    let cities_config = vec![
        CityHub {
            city_name: "تهران",
            hubs: vec![
                ("ونک", 51.403, 35.757),
                ("جردن", 51.417, 35.759),
                ("سعادت‌آباد", 51.371, 35.788),
                ("میدان انقلاب", 51.391, 35.700),
                ("خیابان بهشتی", 51.428, 35.731),
                ("میدان آزادی", 51.336, 35.699),
                ("شهرک غرب", 51.365, 35.760),
                ("پارک فناوری پردیس", 51.782, 35.735),
                ("تجریش", 51.424, 35.808),
                ("تهرانپارس", 51.532, 35.733),
            ],
            job_target: 4000,
        },
        CityHub {
            city_name: "اصفهان",
            hubs: vec![
                ("چهارباغ عباسی", 51.666, 32.654),
                ("شهرک علمی و تحقیقاتی", 51.524, 32.719),
                ("خیابان نظر شرقی", 51.652, 32.628),
                ("دروازه شیراز", 51.668, 32.617),
                ("بزرگمهر", 51.695, 32.645),
            ],
            job_target: 1600,
        },
        CityHub {
            city_name: "مشهد",
            hubs: vec![
                ("بلوار سجاد", 59.540, 36.315),
                ("خیابان احمدآباد", 59.570, 36.300),
                ("بلوار وکیل‌آباد", 59.510, 36.330),
                ("میدان جانباز", 59.560, 36.320),
            ],
            job_target: 1600,
        },
        CityHub {
            city_name: "شیراز",
            hubs: vec![
                ("بلوار شهید چمران", 52.520, 29.635),
                ("خیابان عفیف‌آباد", 52.505, 29.620),
                ("بلوار معالی‌آباد", 52.460, 29.680),
                ("قصرالدشت", 52.490, 29.650),
            ],
            job_target: 1200,
        },
        CityHub {
            city_name: "تبریز",
            hubs: vec![
                ("بلوار ایل‌گلی", 46.360, 38.050),
                ("کوی ولیعصر", 46.330, 38.075),
                ("میدان آبرسان", 46.295, 38.070),
                ("خیابان شریعتی", 46.280, 38.065),
            ],
            job_target: 900,
        },
        CityHub {
            city_name: "کرج",
            hubs: vec![
                ("جهانشهر", 50.991, 35.832),
                ("گوهردشت", 50.970, 35.850),
                ("بلوار طالقانی", 50.985, 35.820),
            ],
            job_target: 400,
        },
        CityHub {
            city_name: "رشت",
            hubs: vec![
                ("گلسار", 49.583, 37.280),
                ("بلوار دیلمان", 49.595, 37.295),
                ("میدان شهرداری", 49.580, 37.275),
            ],
            job_target: 300,
        },
    ];

    let job_titles = [
        "توسعه‌دهنده ارشد Rust", "برنامه‌نویس React و Next.js", "مهندس دواپس (Kubernetes & CI/CD)",
        "مدیر ارشد محصول (Product Lead)", "طراح تجربه کاربری (UI/UX Designer)", "برنامه‌نویس بک‌اند Go",
        "توسعه‌دهنده پایتون و هوش مصنوعی", "حسابدار مالی و مالیاتی", "کارشناس سئو و رشد مارکتینگ",
        "مدیر پروژه چابک (Scrum Master)", "مهندس کلان‌داده و پایگاه‌داده PostGIS", "کارشناس کنترل کیفیت نرم‌افزار (QA)"
    ];

    let workplace_types = ["onsite", "hybrid", "remote"];
    let exp_levels = ["junior", "mid_level", "senior", "lead"];

    let mut rng = rand::thread_rng();

    // ایجاد حدود ۲,۵۰۰ نقطه مکانی
    tracing::info!("Generating distinct physical locations across all 7 cities...");
    let mut location_ids = Vec::new();
    let mut loc_lons = Vec::new();
    let mut loc_lats = Vec::new();
    let mut loc_addresses = Vec::new();

    let mut city_location_map: Vec<(&'static str, Vec<Uuid>)> = Vec::new();

    for city in &cities_config {
        let num_locs = (city.job_target / 4).max(10);
        let mut city_loc_ids = Vec::new();

        for i in 0..num_locs {
            let hub = city.hubs[rng.gen_range(0..city.hubs.len())];
            let lon_offset: f64 = rng.gen_range(-0.022..0.022);
            let lat_offset: f64 = rng.gen_range(-0.022..0.022);
            let lon = hub.1 + lon_offset;
            let lat = hub.2 + lat_offset;

            let lid = Uuid::new_v4();
            location_ids.push(lid);
            loc_lons.push(lon);
            loc_lats.push(lat);
            loc_addresses.push(format!("{}، محدوده {}، خیابان دانش پلاک {}", city.city_name, hub.0, (i % 80) + 1));
            city_loc_ids.push(lid);
        }
        city_location_map.push((city.city_name, city_loc_ids));
    }

    // Bulk Insert لوکیشن‌ها با UNNEST
    sqlx::query(r#"
        INSERT INTO locations (id, coordinates, address_summary, precision, source, source_id)
        SELECT 
            u.id,
            ST_SetSRID(ST_MakePoint(u.lon, u.lat), 4326),
            u.addr,
            'exact',
            'opportunity',
            'bulk_10k_' || u.id
        FROM UNNEST(
            $1::uuid[],
            $2::float8[],
            $3::float8[],
            $4::text[]
        ) AS u(id, lon, lat, addr)
    "#)
    .bind(&location_ids)
    .bind(&loc_lons)
    .bind(&loc_lats)
    .bind(&loc_addresses)
    .execute(&mut *tx)
    .await?;

    tracing::info!("Inserted {} physical locations into PostGIS.", location_ids.len());

    // تولید و اینسرت ۱۰,۰۰۰ موقعیت شغلی
    let total_target = 10000;
    let mut generated_count = 0;

    for (city_idx, city) in cities_config.iter().enumerate() {
        let city_locs = &city_location_map[city_idx].1;
        let count_for_city = city.job_target;

        let mut opp_ids = Vec::with_capacity(count_for_city);
        let mut comp_ids = Vec::with_capacity(count_for_city);
        let mut titles = Vec::with_capacity(count_for_city);
        let mut descriptions = Vec::with_capacity(count_for_city);
        let mut c_ids = Vec::with_capacity(count_for_city);
        let mut wps = Vec::with_capacity(count_for_city);
        let mut exps = Vec::with_capacity(count_for_city);
        let mut s_mins = Vec::with_capacity(count_for_city);
        let mut s_maxs = Vec::with_capacity(count_for_city);

        let mut rel_opp_ids = Vec::with_capacity(count_for_city);
        let mut rel_loc_ids = Vec::with_capacity(count_for_city);

        for _ in 0..count_for_city {
            let opp_id = Uuid::new_v4();
            let comp_id = company_ids[rng.gen_range(0..company_ids.len())];
            let base_title = job_titles[rng.gen_range(0..job_titles.len())];
            let wp = workplace_types[rng.gen_range(0..workplace_types.len())];
            let exp = exp_levels[rng.gen_range(0..exp_levels.len())];
            let cat_id = cat_ids[rng.gen_range(0..cat_ids.len())];
            let loc_id = city_locs[rng.gen_range(0..city_locs.len())];

            let min_s = rng.gen_range(25..85) * 1_000_000;
            let max_s = min_s + rng.gen_range(15..40) * 1_000_000;

            opp_ids.push(opp_id);
            comp_ids.push(comp_id);
            titles.push(format!("{} ({})", base_title, city.city_name));
            descriptions.push(format!("فرصت همکاری تمام‌وقت در شرکت پیشرو در {}. تسلط به مهارت‌های مرتبط و روحیه کار تیمی الزامی است.", city.city_name));
            c_ids.push(cat_id);
            wps.push(wp.to_string());
            exps.push(exp.to_string());
            s_mins.push(Decimal::new(min_s, 0));
            s_maxs.push(Decimal::new(max_s, 0));

            if wp != "remote" {
                rel_opp_ids.push(opp_id);
                rel_loc_ids.push(loc_id);
            }
        }

        sqlx::query(r#"
            INSERT INTO opportunities (
                id, company_id, title, description, category_id,
                opportunity_type, workplace_type, remote_scope, experience_level,
                salary_min, salary_max, salary_currency, salary_period,
                status, published_at, expires_at, created_at, updated_at
            )
            SELECT 
                u.id, u.comp_id, u.title, u.desc_text, u.cat_id,
                'full_time', u.wp, 'country', u.exp,
                u.s_min, u.s_max, 'IRR', 'monthly',
                'published', NOW(), NOW() + INTERVAL '90 days', NOW(), NOW()
            FROM UNNEST(
                $1::uuid[],
                $2::uuid[],
                $3::text[],
                $4::text[],
                $5::uuid[],
                $6::text[],
                $7::text[],
                $8::numeric[],
                $9::numeric[]
            ) AS u(id, comp_id, title, desc_text, cat_id, wp, exp, s_min, s_max)
        "#)
        .bind(&opp_ids)
        .bind(&comp_ids)
        .bind(&titles)
        .bind(&descriptions)
        .bind(&c_ids)
        .bind(&wps)
        .bind(&exps)
        .bind(&s_mins)
        .bind(&s_maxs)
        .execute(&mut *tx)
        .await?;

        sqlx::query(r#"
            INSERT INTO opportunity_locations (opportunity_id, location_id)
            SELECT u.opp_id, u.loc_id
            FROM UNNEST($1::uuid[], $2::uuid[]) AS u(opp_id, loc_id)
        "#)
        .bind(&rel_opp_ids)
        .bind(&rel_loc_ids)
        .execute(&mut *tx)
        .await?;

        generated_count += count_for_city;
        tracing::info!("Ingested {} / {} jobs (City: {})...", generated_count, total_target, city.city_name);
    }

    tx.commit().await?;
    tracing::info!("SUCCESS! Ingested 10,000 real opportunities across 7 Iranian cities into PostGIS.");
    Ok(())
}