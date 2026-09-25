use sqlx::PgPool;
use uuid::Uuid;
use rand::Rng;
use rust_decimal::Decimal;

pub async fn clean_seeded_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Cleaning up previous opportunity and location test data...");

    let mut tx = pool.begin().await?;

    sqlx::query(r#"
        TRUNCATE TABLE 
            applications,
            saved_searches,
            saved_opportunities,
            saved_companies,
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
    tracing::info!("Database cleaned successfully!");
    Ok(())
}

pub async fn seed_10k_national_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    clean_seeded_data(pool).await?;

    tracing::info!("Generating 10,000 Multi-Industry National Opportunities across Iran...");

    let mut tx = pool.begin().await?;

    // ۱. دسته‌بندی‌های جامع اصناف و صنایع
    let categories_data = [
        ("رستوران، کافه و قنادی", "restaurant-cafe", "استخدام باریستا، آشپز، سالندار، صندوقدار و پیک"),
        ("فروشگاه، بوتیک و بازار", "retail-shop", "فروشندگی پوشاک، سوپرمارکت، صندوقداری و بازاریابی"),
        ("پزشکی، کلینیک و سلامت", "medical-health", "پزشک، پرستار، منشی مطب، تکنسین داروخانه و فیزیوتراپی"),
        ("سالن زیبایی و آرایشگری", "beauty-wellness", "آرایشگر، ناخن‌کار، میکاپ‌آرتیست و ادمین سالن"),
        ("فنی، صنعتی و کارگاه", "technical-workshop", "تراشکار، جوشکار، مکانیک، سیم‌کشی و تکنسین تولید"),
        ("حسابداری، مالی و اداری", "finance-admin", "حسابدار، کارشناس امور مالیاتی، مسئول دفتر و کارمند اداری"),
        ("فروش، بازاریابی و املاک", "sales-real-estate", "مشاور املاک، کارشناس فروش تلفنی، بازاریاب میدانی"),
        ("فناوری اطلاعات و نرم‌افزار", "software-it", "توسعه‌دهنده وب، هوش مصنوعی، موبایل و پشتیبان شبکه"),
        ("طراحی، گرافیک و تولید محتوا", "design-creative", "طراح گرافیک، عکاس، تدوینگر ویدیو و ادمین سوشال"),
        ("آموزش، تدریس و زبان", "education-teaching", "مدرس زبان انگلیسی، مربی مهدکودک و مشاور تحصیلی"),
        ("حمل‌ونقل، انبار و لجستیک", "logistics-warehouse", "راننده، انباردار، موزع و کارگر بسته‌بندی"),
        ("نگهبانی، خدمات و حراست", "security-services", "نگهبان مجتمع، نیروی خدمات، نظافتچی و لابی‌من"),
    ];

    let mut cat_map = Vec::new();
    for (name, slug, desc) in categories_data {
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
                .bind(desc)
                .fetch_one(&mut *tx)
                .await?
            }
        };

        cat_map.push((slug, cid));
    }

    // ۲. مهارت‌های اصناف مختلف
    let skills_data = [
        "باریستایی و لاته آرت", "آشپزی و تخته‌کاری", "فروشندگی حرفه‌ای", "نرم‌افزار هلو",
        "اصول حسابداری و مالیات", "تزریقات و پانسمان", "میکاپ و گریم", "تراشکاری CNC",
        "مذاکره و ارتباط با مشتری", "فتوشاپ و ایلاستریتور", "تولید محتوا و اینستاگرام",
        "رانندگی با خودرو/موتور", "روابط عمومی بالا", "پایتون", "React", "Rust", "زبان انگلیسی"
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

    // ۳. ایجاد ۵۰ کسب‌وکار واقعی در اصناف مختلف
    struct BusinessTemplate {
        name: &'static str,
        category_slug: &'static str,
        b_type: &'static str,
    }

    let business_templates = [
        BusinessTemplate { name: "کافه رستوران ونک", category_slug: "restaurant-cafe", b_type: "restaurant_cafe" },
        BusinessTemplate { name: "بوتیک پوشاک کارن", category_slug: "retail-shop", b_type: "retail_shop" },
        BusinessTemplate { name: "کلینیک دندانپزشکی مهر", category_slug: "medical-health", b_type: "clinic_office" },
        BusinessTemplate { name: "کارگاه تراشکاری پیشگام", category_slug: "technical-workshop", b_type: "workshop" },
        BusinessTemplate { name: "هایپرمارکت پالادیوم", category_slug: "retail-shop", b_type: "retail_shop" },
        BusinessTemplate { name: "سالن زیبایی و عروس پریا", category_slug: "beauty-wellness", b_type: "clinic_office" },
        BusinessTemplate { name: "گروه مالی و حسابرسی پارس", category_slug: "finance-admin", b_type: "corporate" },
        BusinessTemplate { name: "املاک و سرمایه‌گذاری دلتا", category_slug: "sales-real-estate", b_type: "corporate" },
        BusinessTemplate { name: "شرکت مهندسی داده سپهر", category_slug: "software-it", b_type: "corporate" },
        BusinessTemplate { name: "آکادمی بین‌المللی کیش‌ایر", category_slug: "education-teaching", b_type: "corporate" },
        BusinessTemplate { name: "انبار مرکزی دیجی‌پخش", category_slug: "logistics-warehouse", b_type: "workshop" },
        BusinessTemplate { name: "کافه کتاب فردوسی", category_slug: "restaurant-cafe", b_type: "restaurant_cafe" },
        BusinessTemplate { name: "فروشگاه زنجیره‌ای افق نوین", category_slug: "retail-shop", b_type: "retail_shop" },
        BusinessTemplate { name: "مرکز تصویربرداری نور", category_slug: "medical-health", b_type: "clinic_office" },
    ];

    let mut company_ids = Vec::new();
    let mut comp_cat_map = Vec::new();

    for (i, b) in business_templates.iter().cycle().take(60).enumerate() {
        let comp_id = Uuid::new_v4();
        let slug = format!("biz-{}-{}", i + 1, &comp_id.to_string()[..6]);
        let comp_name = format!("{} (شعبه {})", b.name, (i % 5) + 1);

        sqlx::query(r#"
            INSERT INTO companies (id, name, slug, description, website, business_type, verification_status, created_at, updated_at)
            VALUES ($1, $2, $3, 'کسب‌وکار فعال در سامانه مکان‌محور', 'https://spatijob.ir', $4, 'verified', NOW(), NOW())
        "#)
        .bind(comp_id)
        .bind(&comp_name)
        .bind(&slug)
        .bind(b.b_type)
        .execute(&mut *tx).await?;

        company_ids.push(comp_id);
        comp_cat_map.push((comp_id, b.category_slug));
    }

    // ۴. قطب‌های استانی و محله‌های واقعی
    struct CityHub {
        city_name: &'static str,
        hubs: Vec<(&'static str, f64, f64)>,
        target_count: usize,
    }

    let cities_config = vec![
        CityHub {
            city_name: "تهران",
            hubs: vec![
                ("ونک", 51.403, 35.757),
                ("میدان انقلاب", 51.391, 35.700),
                ("بازار بزرگ تهران", 51.419, 35.672),
                ("سعادت‌آباد", 51.371, 35.788),
                ("تجریش", 51.424, 35.808),
                ("صادقیه", 51.341, 35.722),
                ("تهرانپارس", 51.532, 35.733),
                ("نازی‌آباد", 51.402, 35.642),
                ("شهرک غرب", 51.365, 35.760),
                ("خیابان جمهوری", 51.405, 35.694),
            ],
            target_count: 4500,
        },
        CityHub {
            city_name: "اصفهان",
            hubs: vec![
                ("چهارباغ عباسی", 51.666, 32.654),
                ("خیابان نظر شرقی", 51.652, 32.628),
                ("دروازه شیراز", 51.668, 32.617),
                ("بزرگمهر", 51.695, 32.645),
            ],
            target_count: 1400,
        },
        CityHub {
            city_name: "مشهد",
            hubs: vec![
                ("بلوار سجاد", 59.540, 36.315),
                ("احمدآباد", 59.570, 36.300),
                ("وکیل‌آباد", 59.510, 36.330),
                ("میدان جانباز", 59.560, 36.320),
            ],
            target_count: 1400,
        },
        CityHub {
            city_name: "شیراز",
            hubs: vec![
                ("بلوار چمران", 52.520, 29.635),
                ("عفیف‌آباد", 52.505, 29.620),
                ("معالی‌آباد", 52.460, 29.680),
                ("قصرالدشت", 52.490, 29.650),
            ],
            target_count: 1000,
        },
        CityHub {
            city_name: "تبریز",
            hubs: vec![
                ("ائل‌گلی", 46.360, 38.050),
                ("کوی ولیعصر", 46.330, 38.075),
                ("آبرسان", 46.295, 38.070),
            ],
            target_count: 800,
        },
        CityHub {
            city_name: "کرج",
            hubs: vec![
                ("گوهردشت", 50.970, 35.850),
                ("جهانشهر", 50.991, 35.832),
                ("طالقانی", 50.985, 35.820),
            ],
            target_count: 500,
        },
        CityHub {
            city_name: "رشت",
            hubs: vec![
                ("گلسار", 49.583, 37.280),
                ("بلوار دیلمان", 49.595, 37.295),
                ("میدان شهرداری", 49.580, 37.275),
            ],
            target_count: 400,
        },
    ];

    let job_titles_by_cat = [
        ("restaurant-cafe", vec!["باریستا و بارتندر ماهر", "کمک‌آشپز فست‌فود", "سالندار و ویتر خوش‌برخورد", "صندوقدار کافه رستوران", "پیک موتوری تمام‌وقت"]),
        ("retail-shop", vec!["فروشنده مجرب پوشاک زنانه", "صندوقدار هایپرمارکت", "فروشنده حضوری موبایل و جانبی", "مسئول چیدمان و قفسه", "انباردار فروشگاه"]),
        ("medical-health", vec!["دستیار دندانپزشک مجرب", "تکنسین داروخانه (نسخه‌پیچ)", "منشی و پذیرش مطب پزشک", "پرستار بخش کلینیک", "کارشناس فیزیوتراپی"]),
        ("beauty-wellness", vec!["ناخن‌کار حرفه‌ای با مشتری", "میکاپ‌آرتیست و شینیون‌کار", "ادمین اینستاگرام و نوبت‌دهی سالن", "اصلاح و کوتاهی مو"]),
        ("technical-workshop", vec!["تراشکار و فرزکار ماهر", "جوشکار برق و CO2", "مکانیک و جلوبندی‌ساز", "مونتاژکار خط تولید", "برق‌کار صنعتی"]),
        ("finance-admin", vec!["حسابدار مالی و مالیاتی", "کمک‌حسابدار مسلط به سپیدار", "کارمند اداری و مسئول دفتر", "کارشناس بازرگانی و فروش"]),
        ("sales-real-estate", vec!["مشاور املاک مبتدی و حرفه‌ای", "کارشناس فروش تلفنی", "ویزیتور حضوری با انگیزه", "مدیر قراردادهای تجاری"]),
        ("software-it", vec!["توسعه‌دهنده وب (Fullstack)", "برنامه‌نویس Flutter", "کارشناس پشتیبانی شبکه و Helpdesk", "طراح رابط کاربری UI/UX"]),
        ("design-creative", vec!["طراح گرافیک و مسلط به فتوشاپ", "تدوینگر ویدیو و تیزر تبلیغاتی", "عکاس صنعتی محصولات", "سناریونویس و تولیدکننده محتوا"]),
        ("education-teaching", vec!["مدرس زبان انگلیسی خردسالان", "مربی مهدکودک باانرژی", "مشاور تحصیلی کنکور"]),
        ("logistics-warehouse", vec!["راننده با وانت مسقف", "موزع و پخش‌کننده کالا", "کارگر بسته‌بندی انبار"]),
        ("security-services", vec!["نگهبان و لابی‌من برج مسکونی", "نیروی خدمات و پذیرایی اداری", "سرایدار متعهد با خانواده"]),
    ];

    let shift_hours = [
        "۹ صبح تا ۱۸ عصر",
        "۱۰ صبح تا ۲۱ شب (با استراحت)",
        "۱۶ عصر تا ۲۴ بامداد (شیفت شب)",
        "۹ صبح تا ۱۵ عصر (شیفت صبح)",
        "پاره‌وقت و منعطف",
    ];

    let mut rng = rand::thread_rng();

    // ایجاد حدود ۲,۵۰۰ لوکیشن متنوع در سراسر ایران
    tracing::info!("Generating 2,500 physical branches and locations across Iranian cities...");
    let mut location_ids = Vec::new();
    let mut loc_lons = Vec::new();
    let mut loc_lats = Vec::new();
    let mut loc_addrs = Vec::new();
    let mut city_locs_map: Vec<(&'static str, Vec<Uuid>)> = Vec::new();

    for city in &cities_config {
        let loc_count = (city.target_count / 4).max(20);
        let mut city_lids = Vec::new();

        for i in 0..loc_count {
            let hub = city.hubs[rng.gen_range(0..city.hubs.len())];
            let lon_offset: f64 = rng.gen_range(-0.024..0.024);
            let lat_offset: f64 = rng.gen_range(-0.024..0.024);
            let lon = hub.1 + lon_offset;
            let lat = hub.2 + lat_offset;

            let lid = Uuid::new_v4();
            location_ids.push(lid);
            loc_lons.push(lon);
            loc_lats.push(lat);
            loc_addrs.push(format!("{}، محدوده {}، پلاک {}", city.city_name, hub.0, (i % 60) + 1));
            city_lids.push(lid);
        }
        city_locs_map.push((city.city_name, city_lids));
    }

    // Bulk insert لوکیشن‌ها
    sqlx::query(r#"
        INSERT INTO locations (id, coordinates, address_summary, precision, source, source_id)
        SELECT 
            u.id,
            ST_SetSRID(ST_MakePoint(u.lon, u.lat), 4326),
            u.addr,
            'exact',
            'opportunity',
            'bulk_' || u.id
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
    .bind(&loc_addrs)
    .execute(&mut *tx)
    .await?;

    tracing::info!("Inserted 2,500 real physical locations into PostGIS.");

    // پیوند شعبات به شرکت‌ها
    for (i, cid) in company_ids.iter().enumerate() {
        let lid = location_ids[i % location_ids.len()];
        let _ = sqlx::query(
            r#"
            INSERT INTO company_locations (company_id, location_id, is_headquarters)
            SELECT $1, $2, true
            WHERE NOT EXISTS (
                SELECT 1 FROM company_locations WHERE company_id = $1 AND location_id = $2
            )
            "#
        )
        .bind(cid)
        .bind(lid)
        .execute(&mut *tx).await;
    }

    // تولید ۱۰,۰۰۰ آگهی با نسبت‌های دقیق و واقعی
    let total_target = 10000;
    let mut total_generated = 0;

    for (city_idx, city) in cities_config.iter().enumerate() {
        let city_locs = &city_locs_map[city_idx].1;
        let count = city.target_count;

        let mut opp_ids = Vec::with_capacity(count);
        let mut comp_ids = Vec::with_capacity(count);
        let mut titles = Vec::with_capacity(count);
        let mut descs = Vec::with_capacity(count);
        let mut category_uuids = Vec::with_capacity(count);
        let mut workplace_types = Vec::with_capacity(count);
        let mut exp_levels = Vec::with_capacity(count);
        let mut salary_mins = Vec::with_capacity(count);
        let mut salary_maxs = Vec::with_capacity(count);
        let mut is_urgents = Vec::with_capacity(count);
        let mut is_featureds = Vec::with_capacity(count);
        let mut working_hours_list = Vec::with_capacity(count);
        let mut gender_prefs = Vec::with_capacity(count);
        let mut has_insurances = Vec::with_capacity(count);

        let mut rel_opp_ids = Vec::with_capacity(count);
        let mut rel_loc_ids = Vec::with_capacity(count);

        for _ in 0..count {
            let opp_id = Uuid::new_v4();
            let comp_idx = rng.gen_range(0..company_ids.len());
            let comp_id = company_ids[comp_idx];
            let comp_cat_slug = comp_cat_map[comp_idx].1;

            let cat_titles = job_titles_by_cat
                .iter()
                .find(|(s, _)| *s == comp_cat_slug)
                .map(|(_, list)| list.as_slice())
                .unwrap_or(&["کارشناس امور تجاری", "فروشنده باانگیزه", "کارمند اجرایی"]);

            let base_title = cat_titles[rng.gen_range(0..cat_titles.len())];
            let cat_id = cat_map.iter().find(|(s, _)| *s == comp_cat_slug).map(|(_, id)| *id).unwrap_or(cat_map[0].1);
            let loc_id = city_locs[rng.gen_range(0..city_locs.len())];

            // ۱۵٪ استخدام فوری
            let is_urg = rng.gen_bool(0.15);
            // ۸٪ پین طلایی
            let is_feat = rng.gen_bool(0.08);

            let wp = if comp_cat_slug == "software-it" || comp_cat_slug == "design-creative" {
                match rng.gen_range(0..10) {
                    0..=3 => "remote",
                    4..=6 => "hybrid",
                    _ => "onsite"
                }
            } else {
                "onsite"
            };

            let (min_s, max_s) = match comp_cat_slug {
                "medical-health" | "software-it" => {
                    let s = rng.gen_range(35..90) * 1_000_000;
                    (s, s + rng.gen_range(15..45) * 1_000_000)
                },
                "restaurant-cafe" | "retail-shop" | "beauty-wellness" => {
                    let s = rng.gen_range(16..38) * 1_000_000;
                    (s, s + rng.gen_range(5..15) * 1_000_000)
                },
                _ => {
                    let s = rng.gen_range(18..48) * 1_000_000;
                    (s, s + rng.gen_range(10..20) * 1_000_000)
                }
            };

            let hours = shift_hours[rng.gen_range(0..shift_hours.len())];
            let gender = match rng.gen_range(0..10) {
                0..=6 => "any",
                7..=8 => "female",
                _ => "male"
            };

            opp_ids.push(opp_id);
            comp_ids.push(comp_id);
            titles.push(format!("{} ({})", base_title, city.city_name));
            descs.push(format!("فرصت همکاری در مجموعه معتبر واقع در {}. ساعت کاری: {}. محیط صمیمی و پرداخت به‌موقع.", city.city_name, hours));
            category_uuids.push(cat_id);
            workplace_types.push(wp.to_string());
            exp_levels.push("mid_level".to_string());
            salary_mins.push(Decimal::new(min_s, 0));
            salary_maxs.push(Decimal::new(max_s, 0));
            is_urgents.push(is_urg);
            is_featureds.push(is_feat);
            working_hours_list.push(hours.to_string());
            gender_prefs.push(gender.to_string());
            has_insurances.push(rng.gen_bool(0.85));

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
                status, is_urgent, is_featured, working_hours, gender_preference, has_insurance,
                published_at, expires_at, created_at, updated_at
            )
            SELECT 
                u.id, u.comp_id, u.title, u.desc_text, u.cat_id,
                'full_time', u.wp, 'country', u.exp,
                u.s_min, u.s_max, 'IRR', 'monthly',
                'published', u.is_urg, u.is_feat, u.hours, u.gender, u.ins,
                NOW() - (random() * INTERVAL '15 days'), NOW() + INTERVAL '45 days', NOW(), NOW()
            FROM UNNEST(
                $1::uuid[],
                $2::uuid[],
                $3::text[],
                $4::text[],
                $5::uuid[],
                $6::text[],
                $7::text[],
                $8::numeric[],
                $9::numeric[],
                $10::boolean[],
                $11::boolean[],
                $12::text[],
                $13::text[],
                $14::boolean[]
            ) AS u(id, comp_id, title, desc_text, cat_id, wp, exp, s_min, s_max, is_urg, is_feat, hours, gender, ins)
        "#)
        .bind(&opp_ids)
        .bind(&comp_ids)
        .bind(&titles)
        .bind(&descs)
        .bind(&category_uuids)
        .bind(&workplace_types)
        .bind(&exp_levels)
        .bind(&salary_mins)
        .bind(&salary_maxs)
        .bind(&is_urgents)
        .bind(&is_featureds)
        .bind(&working_hours_list)
        .bind(&gender_prefs)
        .bind(&has_insurances)
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

        total_generated += count;
        tracing::info!("Ingested {} / {} real diverse jobs (City: {})...", total_generated, total_target, city.city_name);
    }

    tx.commit().await?;
    tracing::info!("SUCCESS! 10,000 multi-industry opportunities ingested into PostGIS!");
    Ok(())
}