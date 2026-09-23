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
    pub business_type: Option<String>,
    pub trade_license_number: Option<String>,
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
            business_type: self.business_type.unwrap_or_else(|| "corporate".to_string()),
            trade_license_number: self.trade_license_number,
            verification_status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CompanyLocationDbRow {
    pub location_id: Uuid,
    pub address_summary: Option<String>,
    pub longitude: f64,
    pub latitude: f64,
    pub is_headquarters: bool,
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
            INSERT INTO companies (name, slug, description, website, business_type, trade_license_number, verification_status)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending')
            RETURNING id, name, slug, description, logo_storage_key, website, business_type, trade_license_number, verification_status, created_at, updated_at
        "#;

        let comp_row = sqlx::query_as::<_, CompanyDbRow>(comp_sql)
            .bind(&new_company.name)
            .bind(&new_company.slug)
            .bind(&new_company.description)
            .bind(&new_company.website)
            .bind(&new_company.business_type)
            .bind(&new_company.trade_license_number)
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

    pub async fn list_user_companies(&self, user_id: Uuid) -> Result<Vec<(Company, CompanyRole)>, StorageError> {
        #[derive(sqlx::FromRow)]
        struct UserCompanyDbRow {
            pub id: Uuid,
            pub name: String,
            pub slug: String,
            pub description: Option<String>,
            pub logo_storage_key: Option<String>,
            pub website: Option<String>,
            pub business_type: Option<String>,
            pub trade_license_number: Option<String>,
            pub verification_status: String,
            pub created_at: DateTime<Utc>,
            pub updated_at: DateTime<Utc>,
            pub role: String,
        }

        let sql = r#"
            SELECT 
                c.id, c.name, c.slug, c.description, c.logo_storage_key, c.website,
                c.business_type, c.trade_license_number,
                c.verification_status, c.created_at, c.updated_at, cm.role
            FROM companies c
            INNER JOIN company_memberships cm ON cm.company_id = c.id
            WHERE cm.user_id = $1
            ORDER BY cm.created_at ASC
        "#;

        let rows = sqlx::query_as::<_, UserCompanyDbRow>(sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| {
            let role = CompanyRole::from_str(&r.role)?;
            let comp = CompanyDbRow {
                id: r.id,
                name: r.name,
                slug: r.slug,
                description: r.description,
                logo_storage_key: r.logo_storage_key,
                website: r.website,
                business_type: r.business_type,
                trade_license_number: r.trade_license_number,
                verification_status: r.verification_status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            }.to_domain();
            Some((comp, role))
        }).collect())
    }

    pub async fn update_company(
        &self,
        company_id: Uuid,
        name: &str,
        slug: &str,
        description: Option<&str>,
        website: Option<&str>,
        logo_storage_key: Option<&str>,
    ) -> Result<Company, StorageError> {
        let sql = r#"
            UPDATE companies
            SET name = $2,
                slug = $3,
                description = $4,
                website = $5,
                logo_storage_key = COALESCE($6, logo_storage_key),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, slug, description, logo_storage_key, website, business_type, trade_license_number, verification_status, created_at, updated_at
        "#;

        let row = sqlx::query_as::<_, CompanyDbRow>(sql)
            .bind(company_id)
            .bind(name)
            .bind(slug)
            .bind(description)
            .bind(website)
            .bind(logo_storage_key)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.to_domain())
    }

    pub async fn find_by_id(&self, company_id: Uuid) -> Result<Option<Company>, StorageError> {
        let sql = "SELECT * FROM companies WHERE id = $1";
        let row: Option<CompanyDbRow> = sqlx::query_as(sql)
            .bind(company_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(CompanyDbRow::to_domain))
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Company>, StorageError> {
        let sql = "SELECT * FROM companies WHERE LOWER(slug) = LOWER($1)";
        let row: Option<CompanyDbRow> = sqlx::query_as(sql)
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(CompanyDbRow::to_domain))
    }

    pub async fn get_user_role(&self, company_id: Uuid, user_id: Uuid) -> Result<Option<CompanyRole>, StorageError> {
        let sql = "SELECT role FROM company_memberships WHERE company_id = $1 AND user_id = $2";
        let role_str: Option<String> = sqlx::query_scalar(sql).bind(company_id).bind(user_id).fetch_optional(&self.pool).await?;
        Ok(role_str.as_deref().and_then(CompanyRole::from_str))
    }

    pub async fn add_company_location(&self, company_id: Uuid, location_id: Uuid, is_headquarters: bool) -> Result<(), StorageError> {
        let sql = r#"
            INSERT INTO company_locations (company_id, location_id, is_headquarters)
            VALUES ($1, $2, $3)
            ON CONFLICT (company_id, location_id) DO UPDATE SET is_headquarters = EXCLUDED.is_headquarters
        "#;
        sqlx::query(sql).bind(company_id).bind(location_id).bind(is_headquarters).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_company_locations(&self, company_id: Uuid) -> Result<Vec<CompanyLocationDbRow>, StorageError> {
        let sql = r#"
            SELECT 
                cl.location_id,
                loc.address_summary,
                ST_X(loc.coordinates::geometry) AS longitude,
                ST_Y(loc.coordinates::geometry) AS latitude,
                cl.is_headquarters
            FROM company_locations cl
            INNER JOIN locations loc ON loc.id = cl.location_id
            WHERE cl.company_id = $1
            ORDER BY cl.is_headquarters DESC, cl.created_at ASC
        "#;
        let rows = sqlx::query_as::<_, CompanyLocationDbRow>(sql).bind(company_id).fetch_all(&self.pool).await?;
        Ok(rows)
    }

    pub async fn list_members(&self, company_id: Uuid) -> Result<Vec<CompanyMembership>, StorageError> {
        #[derive(sqlx::FromRow)]
        struct MemberDbRow {
            id: Uuid,
            company_id: Uuid,
            user_id: Uuid,
            role: String,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let sql = "SELECT * FROM company_memberships WHERE company_id = $1 ORDER BY created_at ASC";
        let rows = sqlx::query_as::<_, MemberDbRow>(sql).bind(company_id).fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                CompanyRole::from_str(&r.role).map(|role| CompanyMembership {
                    id: r.id,
                    company_id: r.company_id,
                    user_id: r.user_id,
                    role,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
            })
            .collect())
    }

    pub async fn add_member(&self, company_id: Uuid, user_id: Uuid, role: CompanyRole) -> Result<(), StorageError> {
        let sql = r#"
            INSERT INTO company_memberships (company_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (company_id, user_id) DO UPDATE SET role = EXCLUDED.role, updated_at = NOW()
        "#;
        sqlx::query(sql).bind(company_id).bind(user_id).bind(role.as_str()).execute(&self.pool).await?;
        Ok(())
    }
}