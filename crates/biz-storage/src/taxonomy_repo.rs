use crate::error::StorageError;
use biz_domain::taxonomy::{Category, CityTaxonomy, Country, Industry, Occupation, Skill};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct CategoryDbRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CategoryDbRow {
    fn to_domain(self) -> Category {
        Category {
            id: self.id,
            name: self.name,
            slug: self.slug,
            description: self.description,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct IndustryDbRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IndustryDbRow {
    fn to_domain(self) -> Industry {
        Industry {
            id: self.id,
            name: self.name,
            slug: self.slug,
            description: self.description,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct OccupationDbRow {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl OccupationDbRow {
    fn to_domain(self) -> Occupation {
        Occupation {
            id: self.id,
            name: self.name,
            code: self.code,
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SkillDbRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl SkillDbRow {
    fn to_domain(self) -> Skill {
        Skill {
            id: self.id,
            name: self.name,
            created_at: self.created_at,
        }
    }
}

#[derive(Clone)]
pub struct TaxonomyRepository {
    pool: PgPool,
}

impl TaxonomyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_categories(&self) -> Result<Vec<Category>, StorageError> {
        let sql = "SELECT id, name, slug, description, created_at, updated_at FROM categories ORDER BY name ASC";
        let rows = sqlx::query_as::<_, CategoryDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(CategoryDbRow::to_domain).collect())
    }

    pub async fn list_industries(&self) -> Result<Vec<Industry>, StorageError> {
        let sql = "SELECT id, name, slug, description, created_at, updated_at FROM industries ORDER BY name ASC";
        let rows = sqlx::query_as::<_, IndustryDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(IndustryDbRow::to_domain).collect())
    }

    pub async fn list_occupations(&self) -> Result<Vec<Occupation>, StorageError> {
        let sql = "SELECT id, name, code, created_at FROM occupations ORDER BY name ASC";
        let rows = sqlx::query_as::<_, OccupationDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(OccupationDbRow::to_domain).collect())
    }

    pub async fn resolve_skill(&self, raw_term: &str) -> Result<Option<Skill>, StorageError> {
        let term = raw_term.trim().to_lowercase();

        let sql = r#"
            SELECT s.id, s.name, s.created_at
            FROM skills s
            WHERE LOWER(s.name) = $1
            UNION
            SELECT s.id, s.name, s.created_at
            FROM skills s
            INNER JOIN skill_aliases a ON a.skill_id = s.id
            WHERE LOWER(a.alias) = $1
            LIMIT 1
        "#;

        let skill_row = sqlx::query_as::<_, SkillDbRow>(sql)
            .bind(term)
            .fetch_optional(&self.pool)
            .await?;

        Ok(skill_row.map(SkillDbRow::to_domain))
    }

    // لیست رسمی و استاندارد کشورها
    pub fn list_countries(&self) -> Vec<Country> {
        vec![
            Country { code: "IRN".into(), name: "ایران".into(), name_en: "Iran".into() },
            Country { code: "AFG".into(), name: "افغانستان".into(), name_en: "Afghanistan".into() },
            Country { code: "IRQ".into(), name: "عراق".into(), name_en: "Iraq".into() },
            Country { code: "TUR".into(), name: "ترکیه".into(), name_en: "Turkey".into() },
            Country { code: "PAK".into(), name: "پاکستان".into(), name_en: "Pakistan".into() },
            Country { code: "SYR".into(), name: "سوریه".into(), name_en: "Syria".into() },
            Country { code: "DEU".into(), name: "آلمان".into(), name_en: "Germany".into() },
            Country { code: "CAN".into(), name: "کانادا".into(), name_en: "Canada".into() },
            Country { code: "ARE".into(), name: "امارات متحده عربی".into(), name_en: "United Arab Emirates".into() },
            Country { code: "OMN".into(), name: "عمان".into(), name_en: "Oman".into() },
            Country { code: "QAT".into(), name: "قطر".into(), name_en: "Qatar".into() },
            Country { code: "RUS".into(), name: "روسیه".into(), name_en: "Russia".into() },
        ]
    }

    // لیست شهرهای رسمی به همراه استان و مختصات جغرافیایی
    pub async fn list_cities(&self) -> Result<Vec<CityTaxonomy>, StorageError> {
        Ok(vec![
            CityTaxonomy { id: Uuid::new_v4(), name: "تهران".into(), province: "تهران".into(), center: [51.3890, 35.7200] },
            CityTaxonomy { id: Uuid::new_v4(), name: "اصفهان".into(), province: "اصفهان".into(), center: [51.6660, 32.6546] },
            CityTaxonomy { id: Uuid::new_v4(), name: "مشهد".into(), province: "خراسان رضوی".into(), center: [59.5700, 36.3000] },
            CityTaxonomy { id: Uuid::new_v4(), name: "شیراز".into(), province: "فارس".into(), center: [52.5200, 29.6350] },
            CityTaxonomy { id: Uuid::new_v4(), name: "تبریز".into(), province: "آذربایجان شرقی".into(), center: [46.3600, 38.0500] },
            CityTaxonomy { id: Uuid::new_v4(), name: "کرج".into(), province: "البرز".into(), center: [50.9915, 35.8327] },
            CityTaxonomy { id: Uuid::new_v4(), name: "قم".into(), province: "قم".into(), center: [50.8764, 34.6399] },
            CityTaxonomy { id: Uuid::new_v4(), name: "اهواز".into(), province: "خوزستان".into(), center: [48.6693, 31.3183] },
            CityTaxonomy { id: Uuid::new_v4(), name: "رشت".into(), province: "گیلان".into(), center: [49.5832, 37.2808] },
            CityTaxonomy { id: Uuid::new_v4(), name: "کرمانشاه".into(), province: "کرمانشاه".into(), center: [47.0650, 34.3277] },
            CityTaxonomy { id: Uuid::new_v4(), name: "یزد".into(), province: "یزد".into(), center: [54.3569, 31.8974] },
            CityTaxonomy { id: Uuid::new_v4(), name: "ارومیه".into(), province: "آذربایجان غربی".into(), center: [45.0761, 37.5527] },
            CityTaxonomy { id: Uuid::new_v4(), name: "همدان".into(), province: "همدان".into(), center: [48.5146, 34.7989] },
            CityTaxonomy { id: Uuid::new_v4(), name: "کرمان".into(), province: "کرمان".into(), center: [57.0788, 30.2839] },
            CityTaxonomy { id: Uuid::new_v4(), name: "بندرعباس".into(), province: "هرمزگان".into(), center: [56.2808, 27.1832] },
        ])
    }
}