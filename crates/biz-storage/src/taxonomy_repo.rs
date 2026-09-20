use crate::error::StorageError;
use biz_domain::taxonomy::{Category, Industry, Occupation, Skill};
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

    // Categories
    pub async fn list_categories(&self) -> Result<Vec<Category>, StorageError> {
        let sql = "SELECT id, name, slug, description, created_at, updated_at FROM categories ORDER BY name ASC";
        let rows = sqlx::query_as::<_, CategoryDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(CategoryDbRow::to_domain).collect())
    }

    // Industries
    pub async fn list_industries(&self) -> Result<Vec<Industry>, StorageError> {
        let sql = "SELECT id, name, slug, description, created_at, updated_at FROM industries ORDER BY name ASC";
        let rows = sqlx::query_as::<_, IndustryDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(IndustryDbRow::to_domain).collect())
    }

    // Occupations
    pub async fn list_occupations(&self) -> Result<Vec<Occupation>, StorageError> {
        let sql = "SELECT id, name, code, created_at FROM occupations ORDER BY name ASC";
        let rows = sqlx::query_as::<_, OccupationDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(OccupationDbRow::to_domain).collect())
    }

    // Skills & Alias Resolution
    /// Resolves an arbitrary skill name or alias to its canonical Skill (Section 27)
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
}