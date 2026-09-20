use crate::error::StorageError;
use biz_domain::company::{Company, CompanyMembership, CompanyRole, CompanyVerificationStatus, NewCompany};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct CompanyDbRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_storage_key: Option<String>,
    pub website: Option<String>,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CompanyDbRow {
    fn to_domain(self) -> Company {
        let verification_status = match self.verification_status.as_str() {
            "pending" => CompanyVerificationStatus::Pending,
            "under_review" => CompanyVerificationStatus::UnderReview,
            "verified" => CompanyVerificationStatus::Verified,
            "rejected" => CompanyVerificationStatus::Rejected,
            "expired" => CompanyVerificationStatus::Expired,
            _ => CompanyVerificationStatus::NotStarted,
        };

        Company {
            id: self.id,
            name: self.name,
            slug: self.slug,
            description: self.description,
            logo_storage_key: self.logo_storage_key,
            website: self.website,
            verification_status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct CompanyRepository {
    pool: PgPool,
}

impl CompanyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_company(
        &self,
        creator_user_id: Uuid,
        new_company: &NewCompany,
    ) -> Result<(Company, CompanyMembership), StorageError> {
        let mut tx = self.pool.begin().await?;

        let comp_sql = r#"
            INSERT INTO companies (name, slug, description, website)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, slug, description, logo_storage_key, website, verification_status, created_at, updated_at
        "#;

        let comp_row = sqlx::query_as::<_, CompanyDbRow>(comp_sql)
            .bind(&new_company.name)
            .bind(&new_company.slug)
            .bind(&new_company.description)
            .bind(&new_company.website)
            .fetch_one(&mut *tx)
            .await?;

        let member_sql = r#"
            INSERT INTO company_memberships (company_id, user_id, role)
            VALUES ($1, $2, 'owner')
            RETURNING id, company_id, user_id, role, created_at, updated_at
        "#;

        #[derive(sqlx::FromRow)]
        struct MemberDbRow {
            id: Uuid,
            company_id: Uuid,
            user_id: Uuid,
            role: String,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let member_row = sqlx::query_as::<_, MemberDbRow>(member_sql)
            .bind(comp_row.id)
            .bind(creator_user_id)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        let company = comp_row.to_domain();
        let membership = CompanyMembership {
            id: member_row.id,
            company_id: member_row.company_id,
            user_id: member_row.user_id,
            role: CompanyRole::Owner,
            created_at: member_row.created_at,
            updated_at: member_row.updated_at,
        };

        Ok((company, membership))
    }

    pub async fn find_by_id(&self, company_id: Uuid) -> Result<Option<Company>, StorageError> {
        let sql = r#"
            SELECT id, name, slug, description, logo_storage_key, website, verification_status, created_at, updated_at
            FROM companies
            WHERE id = $1
        "#;

        let row: Option<CompanyDbRow> = sqlx::query_as(sql)
            .bind(company_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(CompanyDbRow::to_domain))
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Company>, StorageError> {
        let sql = r#"
            SELECT id, name, slug, description, logo_storage_key, website, verification_status, created_at, updated_at
            FROM companies
            WHERE LOWER(slug) = LOWER($1)
        "#;

        let row: Option<CompanyDbRow> = sqlx::query_as(sql)
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(CompanyDbRow::to_domain))
    }

    pub async fn get_user_role(
        &self,
        company_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<CompanyRole>, StorageError> {
        let sql = r#"
            SELECT role
            FROM company_memberships
            WHERE company_id = $1 AND user_id = $2
        "#;

        let role_str: Option<String> = sqlx::query_scalar(sql)
            .bind(company_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(role_str.as_deref().and_then(CompanyRole::from_str))
    }

    pub async fn add_company_location(
        &self,
        company_id: Uuid,
        location_id: Uuid,
        is_headquarters: bool,
    ) -> Result<(), StorageError> {
        let sql = r#"
            INSERT INTO company_locations (company_id, location_id, is_headquarters)
            VALUES ($1, $2, $3)
            ON CONFLICT (company_id, location_id) DO UPDATE SET is_headquarters = EXCLUDED.is_headquarters
        "#;

        sqlx::query(sql)
            .bind(company_id)
            .bind(location_id)
            .bind(is_headquarters)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}