use crate::error::StorageError;
use biz_domain::candidate::{
    Candidate, CandidateEducation, CandidateExperience, CandidateLanguage, CandidateReference,
    CandidateResume, JobInvitation, TalentSearchResult,
};
use biz_domain::saved::CandidatePreferences;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct CandidateDbRow {
    id: Uuid,
    user_id: Uuid,
    first_name: String,
    last_name: String,
    headline: Option<String>,
    bio: Option<String>,
    avatar_storage_key: Option<String>,
    residence_location_id: Option<Uuid>,
    preferred_city: Option<String>,
    preferred_commute_center_id: Option<Uuid>,
    preferred_commute_radius_meters: Option<i32>,
    show_exact_location_to_employers: Option<bool>,
    is_foreign_national: bool,
    nationality_country_code: Option<String>,
    has_disability: bool,
    disability_type: Option<String>,
    gender: Option<String>,
    military_service_status: Option<String>,
    marital_status: Option<String>,
    birth_date: Option<NaiveDate>,
    preferred_category_ids: Vec<Uuid>,
    linkedin_url: Option<String>,
    github_url: Option<String>,
    website_url: Option<String>,
    audio_intro_storage_key: Option<String>,
    job_search_status: Option<String>,
    awards: serde_json::Value,
    certifications: serde_json::Value,
    academic_projects: serde_json::Value,
    publications: serde_json::Value,
    volunteering: serde_json::Value,
    portfolio_items: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CandidateDbRow {
    fn to_domain(self) -> Candidate {
        Candidate {
            id: self.id,
            user_id: self.user_id,
            first_name: self.first_name,
            last_name: self.last_name,
            headline: self.headline,
            bio: self.bio,
            avatar_storage_key: self.avatar_storage_key,
            residence_location_id: self.residence_location_id,
            preferred_city: self.preferred_city,
            preferred_commute_center_id: self.preferred_commute_center_id,
            preferred_commute_radius_meters: self.preferred_commute_radius_meters,
            show_exact_location_to_employers: self.show_exact_location_to_employers.unwrap_or(false),
            is_foreign_national: self.is_foreign_national,
            nationality_country_code: self.nationality_country_code,
            has_disability: self.has_disability,
            disability_type: self.disability_type,
            gender: self.gender,
            military_service_status: self.military_service_status,
            marital_status: self.marital_status,
            birth_date: self.birth_date,
            preferred_category_ids: self.preferred_category_ids,
            linkedin_url: self.linkedin_url,
            github_url: self.github_url,
            website_url: self.website_url,
            audio_intro_storage_key: self.audio_intro_storage_key,
            job_search_status: self.job_search_status.unwrap_or_else(|| "actively_looking".to_string()),
            awards: self.awards,
            certifications: self.certifications,
            academic_projects: self.academic_projects,
            publications: self.publications,
            volunteering: self.volunteering,
            portfolio_items: self.portfolio_items,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ExperienceDbRow {
    id: Uuid,
    candidate_id: Uuid,
    title: String,
    company_name: String,
    activity_field: Option<String>,
    seniority_level: Option<String>,
    company_industry: Option<String>,
    country: Option<String>,
    city: Option<String>,
    start_month: Option<i16>,
    start_year: Option<i32>,
    end_month: Option<i16>,
    end_year: Option<i32>,
    is_current: bool,
    achievements: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct EducationDbRow {
    id: Uuid,
    candidate_id: Uuid,
    institution: String,
    degree_level: String,
    field_of_study: Option<String>,
    gpa: Option<Decimal>,
    start_year: Option<i32>,
    end_year: Option<i32>,
    is_current: bool,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct LanguageDbRow {
    id: Uuid,
    candidate_id: Uuid,
    language_name: String,
    proficiency_level: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct ReferenceDbRow {
    id: Uuid,
    candidate_id: Uuid,
    full_name: String,
    organization_name: String,
    job_title: String,
    relationship_type: Option<String>,
    start_year: Option<i32>,
    end_year: Option<i32>,
    is_still_colleagues: bool,
    phone: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct ResumeDbRow {
    id: Uuid,
    candidate_id: Uuid,
    storage_key: String,
    filename: String,
    mime_type: String,
    file_size: i64,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CandidateMatchContext {
    pub candidate_id: Uuid,
    pub skill_ids: Vec<Uuid>,
    pub preferred_category_ids: Vec<Uuid>,
    pub preferred_city: Option<String>,
    pub commute_coords: Option<(f64, f64)>,
    pub commute_radius_meters: i32,
    pub preferred_workplace_types: Vec<String>,
    pub expected_salary_min: Option<Decimal>,
}

#[derive(Clone)]
pub struct CandidateRepository {
    pool: PgPool,
}

impl CandidateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Candidate>, StorageError> {
        let sql = "SELECT * FROM candidates WHERE user_id = $1";
        let row: Option<CandidateDbRow> = sqlx::query_as(sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(CandidateDbRow::to_domain))
    }

    pub async fn find_by_id(&self, candidate_id: Uuid) -> Result<Option<Candidate>, StorageError> {
        let sql = "SELECT * FROM candidates WHERE id = $1";
        let row: Option<CandidateDbRow> = sqlx::query_as(sql)
            .bind(candidate_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(CandidateDbRow::to_domain))
    }

    pub async fn upsert_profile(&self, item: &Candidate) -> Result<Candidate, StorageError> {
        let sql = r#"
            INSERT INTO candidates (
                user_id, first_name, last_name, headline, bio, preferred_city,
                residence_location_id, preferred_commute_radius_meters, show_exact_location_to_employers,
                is_foreign_national, nationality_country_code, has_disability, disability_type,
                gender, military_service_status, marital_status, birth_date, preferred_category_ids,
                linkedin_url, github_url, website_url, audio_intro_storage_key, job_search_status,
                awards, certifications, academic_projects, publications, volunteering, portfolio_items,
                updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, NOW()
            )
            ON CONFLICT (user_id) DO UPDATE SET
                first_name = EXCLUDED.first_name,
                last_name = EXCLUDED.last_name,
                headline = EXCLUDED.headline,
                bio = EXCLUDED.bio,
                preferred_city = EXCLUDED.preferred_city,
                residence_location_id = EXCLUDED.residence_location_id,
                preferred_commute_radius_meters = EXCLUDED.preferred_commute_radius_meters,
                show_exact_location_to_employers = EXCLUDED.show_exact_location_to_employers,
                is_foreign_national = EXCLUDED.is_foreign_national,
                nationality_country_code = EXCLUDED.nationality_country_code,
                has_disability = EXCLUDED.has_disability,
                disability_type = EXCLUDED.disability_type,
                gender = EXCLUDED.gender,
                military_service_status = EXCLUDED.military_service_status,
                marital_status = EXCLUDED.marital_status,
                birth_date = EXCLUDED.birth_date,
                preferred_category_ids = EXCLUDED.preferred_category_ids,
                linkedin_url = EXCLUDED.linkedin_url,
                github_url = EXCLUDED.github_url,
                website_url = EXCLUDED.website_url,
                audio_intro_storage_key = EXCLUDED.audio_intro_storage_key,
                job_search_status = EXCLUDED.job_search_status,
                awards = EXCLUDED.awards,
                certifications = EXCLUDED.certifications,
                academic_projects = EXCLUDED.academic_projects,
                publications = EXCLUDED.publications,
                volunteering = EXCLUDED.volunteering,
                portfolio_items = EXCLUDED.portfolio_items,
                updated_at = NOW()
            RETURNING *
        "#;

        let row = sqlx::query_as::<_, CandidateDbRow>(sql)
            .bind(item.user_id)
            .bind(&item.first_name)
            .bind(&item.last_name)
            .bind(&item.headline)
            .bind(&item.bio)
            .bind(&item.preferred_city)
            .bind(item.residence_location_id)
            .bind(item.preferred_commute_radius_meters)
            .bind(item.show_exact_location_to_employers)
            .bind(item.is_foreign_national)
            .bind(&item.nationality_country_code)
            .bind(item.has_disability)
            .bind(&item.disability_type)
            .bind(&item.gender)
            .bind(&item.military_service_status)
            .bind(&item.marital_status)
            .bind(item.birth_date)
            .bind(&item.preferred_category_ids)
            .bind(&item.linkedin_url)
            .bind(&item.github_url)
            .bind(&item.website_url)
            .bind(&item.audio_intro_storage_key)
            .bind(&item.job_search_status)
            .bind(&item.awards)
            .bind(&item.certifications)
            .bind(&item.academic_projects)
            .bind(&item.publications)
            .bind(&item.volunteering)
            .bind(&item.portfolio_items)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.to_domain())
    }

    pub async fn get_preferred_coordinates(&self, user_id: Uuid) -> Result<Option<(f64, f64)>, StorageError> {
        let loc_sql = r#"
            SELECT 
                ST_X(loc.coordinates::geometry) AS longitude,
                ST_Y(loc.coordinates::geometry) AS latitude
            FROM candidates c
            INNER JOIN locations loc ON loc.id = COALESCE(c.preferred_commute_center_id, c.residence_location_id)
            WHERE c.user_id = $1
            LIMIT 1
        "#;

        #[derive(sqlx::FromRow)]
        struct CoordsRow {
            longitude: f64,
            latitude: f64,
        }

        let row: Option<CoordsRow> = sqlx::query_as(loc_sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            return Ok(Some((r.longitude, r.latitude)));
        }

        Ok(None)
    }

    pub async fn get_skills_with_names(&self, candidate_id: Uuid) -> Result<Vec<(Uuid, String)>, StorageError> {
        let sql = r#"
            SELECT s.id, s.name 
            FROM candidate_skills cs
            INNER JOIN skills s ON s.id = cs.skill_id
            WHERE cs.candidate_id = $1
            ORDER BY s.name ASC
        "#;

        #[derive(sqlx::FromRow)]
        struct SkillRow {
            id: Uuid,
            name: String,
        }

        let rows = sqlx::query_as::<_, SkillRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| (r.id, r.name)).collect())
    }

    pub async fn get_match_context(&self, user_id: Uuid) -> Result<Option<CandidateMatchContext>, StorageError> {
        let candidate = match self.find_by_user_id(user_id).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        let skills: Vec<Uuid> = sqlx::query_scalar("SELECT skill_id FROM candidate_skills WHERE candidate_id = $1")
            .bind(candidate.id)
            .fetch_all(&self.pool)
            .await?;

        let prefs = self.get_preferences(candidate.id).await?;
        let (workplaces, salary_min) = match prefs {
            Some(p) => (p.preferred_workplace_types, p.expected_salary_min),
            None => (Vec::new(), None),
        };

        let commute_coords = self.get_preferred_coordinates(user_id).await?;
        let commute_radius = candidate.preferred_commute_radius_meters.unwrap_or(5000);

        Ok(Some(CandidateMatchContext {
            candidate_id: candidate.id,
            skill_ids: skills,
            preferred_category_ids: candidate.preferred_category_ids,
            preferred_city: candidate.preferred_city,
            commute_coords,
            commute_radius_meters: commute_radius,
            preferred_workplace_types: workplaces,
            expected_salary_min: salary_min,
        }))
    }

    pub async fn set_skills(&self, candidate_id: Uuid, skill_ids: &[Uuid]) -> Result<(), StorageError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM candidate_skills WHERE candidate_id = $1")
            .bind(candidate_id)
            .execute(&mut *tx)
            .await?;

        for skill_id in skill_ids {
            sqlx::query("INSERT INTO candidate_skills (candidate_id, skill_id) VALUES ($1, $2)")
                .bind(candidate_id)
                .bind(skill_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn add_experience(&self, exp: &CandidateExperience) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO candidate_experiences (
                candidate_id, title, company_name, activity_field, seniority_level,
                company_industry, country, city, start_month, start_year, end_month, end_year,
                is_current, achievements
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id
        "#;

        let id: Uuid = sqlx::query_scalar(sql)
            .bind(exp.candidate_id)
            .bind(&exp.title)
            .bind(&exp.company_name)
            .bind(&exp.activity_field)
            .bind(&exp.seniority_level)
            .bind(&exp.company_industry)
            .bind(&exp.country)
            .bind(&exp.city)
            .bind(exp.start_month)
            .bind(exp.start_year)
            .bind(exp.end_month)
            .bind(exp.end_year)
            .bind(exp.is_current)
            .bind(&exp.achievements)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }

    pub async fn delete_experience(&self, candidate_id: Uuid, exp_id: Uuid) -> Result<(), StorageError> {
        let sql = "DELETE FROM candidate_experiences WHERE id = $1 AND candidate_id = $2";
        sqlx::query(sql)
            .bind(exp_id)
            .bind(candidate_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_experiences(&self, candidate_id: Uuid) -> Result<Vec<CandidateExperience>, StorageError> {
        let sql = "SELECT * FROM candidate_experiences WHERE candidate_id = $1 ORDER BY COALESCE(start_year, 0) DESC";
        let rows = sqlx::query_as::<_, ExperienceDbRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| CandidateExperience {
                id: r.id,
                candidate_id: r.candidate_id,
                title: r.title,
                company_name: r.company_name,
                activity_field: r.activity_field,
                seniority_level: r.seniority_level,
                company_industry: r.company_industry,
                country: r.country,
                city: r.city,
                start_month: r.start_month,
                start_year: r.start_year,
                end_month: r.end_month,
                end_year: r.end_year,
                is_current: r.is_current,
                achievements: r.achievements,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn add_education(&self, edu: &CandidateEducation) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO candidate_educations (
                candidate_id, institution, degree_level, field_of_study, gpa,
                start_year, end_year, is_current
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
        "#;

        let id: Uuid = sqlx::query_scalar(sql)
            .bind(edu.candidate_id)
            .bind(&edu.institution)
            .bind(&edu.degree_level)
            .bind(&edu.field_of_study)
            .bind(edu.gpa)
            .bind(edu.start_year)
            .bind(edu.end_year)
            .bind(edu.is_current)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }

    pub async fn delete_education(&self, candidate_id: Uuid, edu_id: Uuid) -> Result<(), StorageError> {
        let sql = "DELETE FROM candidate_educations WHERE id = $1 AND candidate_id = $2";
        sqlx::query(sql)
            .bind(edu_id)
            .bind(candidate_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_educations(&self, candidate_id: Uuid) -> Result<Vec<CandidateEducation>, StorageError> {
        let sql = "SELECT * FROM candidate_educations WHERE candidate_id = $1 ORDER BY COALESCE(start_year, 0) DESC";
        let rows = sqlx::query_as::<_, EducationDbRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| CandidateEducation {
                id: r.id,
                candidate_id: r.candidate_id,
                institution: r.institution,
                degree_level: r.degree_level,
                field_of_study: r.field_of_study,
                gpa: r.gpa,
                start_year: r.start_year,
                end_year: r.end_year,
                is_current: r.is_current,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn add_language(&self, lang: &CandidateLanguage) -> Result<Uuid, StorageError> {
        let sql = "INSERT INTO candidate_languages (candidate_id, language_name, proficiency_level) VALUES ($1, $2, $3) RETURNING id";
        let id: Uuid = sqlx::query_scalar(sql)
            .bind(lang.candidate_id)
            .bind(&lang.language_name)
            .bind(&lang.proficiency_level)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn delete_language(&self, candidate_id: Uuid, lang_id: Uuid) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM candidate_languages WHERE id = $1 AND candidate_id = $2")
            .bind(lang_id)
            .bind(candidate_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_languages(&self, candidate_id: Uuid) -> Result<Vec<CandidateLanguage>, StorageError> {
        let sql = "SELECT * FROM candidate_languages WHERE candidate_id = $1 ORDER BY language_name ASC";
        let rows = sqlx::query_as::<_, LanguageDbRow>(sql).bind(candidate_id).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| CandidateLanguage {
            id: r.id,
            candidate_id: r.candidate_id,
            language_name: r.language_name,
            proficiency_level: r.proficiency_level,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn add_reference(&self, ref_item: &CandidateReference) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO candidate_references (
                candidate_id, full_name, organization_name, job_title,
                relationship_type, start_year, end_year, is_still_colleagues, phone
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
        "#;
        let id: Uuid = sqlx::query_scalar(sql)
            .bind(ref_item.candidate_id)
            .bind(&ref_item.full_name)
            .bind(&ref_item.organization_name)
            .bind(&ref_item.job_title)
            .bind(&ref_item.relationship_type)
            .bind(ref_item.start_year)
            .bind(ref_item.end_year)
            .bind(ref_item.is_still_colleagues)
            .bind(&ref_item.phone)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn delete_reference(&self, candidate_id: Uuid, ref_id: Uuid) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM candidate_references WHERE id = $1 AND candidate_id = $2")
            .bind(ref_id)
            .bind(candidate_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_references(&self, candidate_id: Uuid) -> Result<Vec<CandidateReference>, StorageError> {
        let sql = "SELECT * FROM candidate_references WHERE candidate_id = $1 ORDER BY full_name ASC";
        let rows = sqlx::query_as::<_, ReferenceDbRow>(sql).bind(candidate_id).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| CandidateReference {
            id: r.id,
            candidate_id: r.candidate_id,
            full_name: r.full_name,
            organization_name: r.organization_name,
            job_title: r.job_title,
            relationship_type: r.relationship_type,
            start_year: r.start_year,
            end_year: r.end_year,
            is_still_colleagues: r.is_still_colleagues,
            phone: r.phone,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn add_resume(&self, res: &CandidateResume) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO candidate_resumes (candidate_id, storage_key, filename, mime_type, file_size)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
        "#;
        let id: Uuid = sqlx::query_scalar(sql)
            .bind(res.candidate_id)
            .bind(&res.storage_key)
            .bind(&res.filename)
            .bind(&res.mime_type)
            .bind(res.file_size)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn delete_resume(&self, candidate_id: Uuid, resume_id: Uuid) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM candidate_resumes WHERE id = $1 AND candidate_id = $2")
            .bind(resume_id)
            .bind(candidate_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_resumes(&self, candidate_id: Uuid) -> Result<Vec<CandidateResume>, StorageError> {
        let sql = "SELECT * FROM candidate_resumes WHERE candidate_id = $1 ORDER BY created_at DESC";
        let rows = sqlx::query_as::<_, ResumeDbRow>(sql).bind(candidate_id).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| CandidateResume {
            id: r.id,
            candidate_id: r.candidate_id,
            storage_key: r.storage_key,
            filename: r.filename,
            mime_type: r.mime_type,
            file_size: r.file_size,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn get_preferences(&self, candidate_id: Uuid) -> Result<Option<CandidatePreferences>, StorageError> {
        #[derive(sqlx::FromRow)]
        struct PrefsDbRow {
            candidate_id: Uuid,
            preferred_workplace_types: Vec<String>,
            preferred_opportunity_types: Vec<String>,
            expected_salary_min: Option<Decimal>,
            salary_currency: String,
            remote_only: bool,
            updated_at: DateTime<Utc>,
        }

        let sql = "SELECT * FROM candidate_preferences WHERE candidate_id = $1";
        let row: Option<PrefsDbRow> = sqlx::query_as(sql)
            .bind(candidate_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| CandidatePreferences {
            candidate_id: r.candidate_id,
            preferred_workplace_types: r.preferred_workplace_types,
            preferred_opportunity_types: r.preferred_opportunity_types,
            expected_salary_min: r.expected_salary_min,
            salary_currency: r.salary_currency,
            remote_only: r.remote_only,
            updated_at: r.updated_at,
        }))
    }

    pub async fn upsert_preferences(
        &self,
        candidate_id: Uuid,
        workplace_types: &[String],
        opportunity_types: &[String],
        expected_salary_min: Option<Decimal>,
        salary_currency: &str,
        remote_only: bool,
    ) -> Result<(), StorageError> {
        let sql = r#"
            INSERT INTO candidate_preferences (
                candidate_id, preferred_workplace_types, preferred_opportunity_types,
                expected_salary_min, salary_currency, remote_only, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (candidate_id) DO UPDATE SET
                preferred_workplace_types = EXCLUDED.preferred_workplace_types,
                preferred_opportunity_types = EXCLUDED.preferred_opportunity_types,
                expected_salary_min = EXCLUDED.expected_salary_min,
                salary_currency = EXCLUDED.salary_currency,
                remote_only = EXCLUDED.remote_only,
                updated_at = NOW()
        "#;

        sqlx::query(sql)
            .bind(candidate_id)
            .bind(workplace_types)
            .bind(opportunity_types)
            .bind(expected_salary_min)
            .bind(salary_currency)
            .bind(remote_only)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn send_invitation(
        &self,
        opp_id: Uuid,
        candidate_id: Uuid,
        company_id: Uuid,
        sender_id: Uuid,
        message: Option<&str>,
    ) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO job_invitations (opportunity_id, candidate_id, company_id, sender_user_id, message, status)
            VALUES ($1, $2, $3, $4, $5, 'pending')
            ON CONFLICT (opportunity_id, candidate_id) DO UPDATE SET
                message = EXCLUDED.message,
                updated_at = NOW()
            RETURNING id
        "#;

        let id: Uuid = sqlx::query_scalar(sql)
            .bind(opp_id)
            .bind(candidate_id)
            .bind(company_id)
            .bind(sender_id)
            .bind(message)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }

    pub async fn search_talents(
        &self,
        q_text: Option<&str>,
        skill_ids: &[Uuid],
        city: Option<&str>,
        actively_looking_only: bool,
        bbox: Option<&geo_types::BoundingBox>,
        center_point: Option<&geo_types::GeoPoint>,
        radius_m: Option<f64>,
        target_opp_id: Option<Uuid>,
        limit: usize,
    ) -> Result<Vec<TalentSearchResult>, StorageError> {
        let (west, south, east, north) = bbox
            .map(|b| (Some(b.west()), Some(b.south()), Some(b.east()), Some(b.north())))
            .unwrap_or((None, None, None, None));

        let (c_lon, c_lat) = center_point
            .map(|p| (Some(p.longitude()), Some(p.latitude())))
            .unwrap_or((None, None));

        let sql = r#"
            WITH candidate_base AS (
                SELECT 
                    c.id AS candidate_id,
                    c.first_name,
                    c.last_name,
                    c.headline,
                    c.bio,
                    c.preferred_city,
                    c.job_search_status,
                    COALESCE(c.preferred_commute_radius_meters, 5000) AS commute_radius_meters,
                    COALESCE(c.show_exact_location_to_employers, false) AS show_exact_location_to_employers,
                    COALESCE(
                        loc.coordinates,
                        CASE 
                            WHEN c.preferred_city = 'تهران' THEN ST_SetSRID(ST_MakePoint(51.3890, 35.7200), 4326)
                            WHEN c.preferred_city = 'اصفهان' THEN ST_SetSRID(ST_MakePoint(51.6660, 32.6546), 4326)
                            WHEN c.preferred_city = 'مشهد' THEN ST_SetSRID(ST_MakePoint(59.5700, 36.3000), 4326)
                            WHEN c.preferred_city = 'شیراز' THEN ST_SetSRID(ST_MakePoint(52.5200, 29.6350), 4326)
                            WHEN c.preferred_city = 'تبریز' THEN ST_SetSRID(ST_MakePoint(46.3600, 38.0500), 4326)
                            WHEN c.preferred_city = 'کرج' THEN ST_SetSRID(ST_MakePoint(50.9900, 35.8300), 4326)
                            ELSE NULL
                        END
                    ) AS base_geom
                FROM candidates c
                LEFT JOIN locations loc ON loc.id = c.residence_location_id
                WHERE c.job_search_status != 'not_looking'
                  AND ($1::text IS NULL OR c.headline ILIKE '%' || $1 || '%' OR c.bio ILIKE '%' || $1 || '%')
                  AND ($2::boolean = false OR c.job_search_status = 'actively_looking')
                  AND ($3::text IS NULL OR c.preferred_city ILIKE '%' || $3 || '%')
                  AND (
                      $4::uuid[] IS NULL OR CARDINALITY($4) = 0 OR
                      EXISTS (
                          SELECT 1 FROM candidate_skills cs
                          WHERE cs.candidate_id = c.id AND cs.skill_id = ANY($4)
                      )
                  )
            ),
            candidate_computed AS (
                SELECT 
                    cb.*,
                    CASE 
                        WHEN cb.show_exact_location_to_employers = true AND cb.base_geom IS NOT NULL THEN
                            ST_X(cb.base_geom::geometry)
                        WHEN cb.base_geom IS NOT NULL THEN
                            ST_X(cb.base_geom::geometry) + (
                                ((('x' || substr(md5(cb.candidate_id::text || 'lon'), 1, 4))::bit(16)::int)::float8 / 65535.0 - 0.5) * 0.025
                            )
                        ELSE NULL
                    END AS display_longitude,
                    CASE 
                        WHEN cb.show_exact_location_to_employers = true AND cb.base_geom IS NOT NULL THEN
                            ST_Y(cb.base_geom::geometry)
                        WHEN cb.base_geom IS NOT NULL THEN
                            ST_Y(cb.base_geom::geometry) + (
                                ((('x' || substr(md5(cb.candidate_id::text || 'lat'), 1, 4))::bit(16)::int)::float8 / 65535.0 - 0.5) * 0.025
                            )
                        ELSE NULL
                    END AS display_latitude,
                    CASE 
                        WHEN $9::float8 IS NOT NULL AND $10::float8 IS NOT NULL AND cb.base_geom IS NOT NULL THEN
                            ST_Distance(cb.base_geom::geography, ST_SetSRID(ST_MakePoint($9, $10), 4326)::geography)
                        ELSE NULL
                    END AS distance_meters,
                    CASE WHEN $13::uuid IS NOT NULL THEN
                        ROUND(
                            (COALESCE(
                                (SELECT COUNT(DISTINCT cs.skill_id)::float8 / NULLIF(COUNT(DISTINCT os.skill_id), 0)
                                 FROM opportunity_skills os
                                 LEFT JOIN candidate_skills cs ON cs.skill_id = os.skill_id AND cs.candidate_id = cb.candidate_id
                                 WHERE os.opportunity_id = $13), 0.20
                            ) * 55.0) +
                            (CASE 
                                WHEN ($9::float8 IS NOT NULL AND cb.base_geom IS NOT NULL AND ST_Distance(cb.base_geom::geography, ST_SetSRID(ST_MakePoint($9, $10), 4326)::geography) <= cb.commute_radius_meters) THEN 30.0
                                WHEN ($9::float8 IS NOT NULL AND cb.base_geom IS NOT NULL AND ST_Distance(cb.base_geom::geography, ST_SetSRID(ST_MakePoint($9, $10), 4326)::geography) <= (cb.commute_radius_meters * 1.5)) THEN 15.0
                                ELSE 5.0
                             END) +
                            (CASE WHEN cb.job_search_status = 'actively_looking' THEN 15.0 ELSE 5.0 END)
                        )::int2
                    ELSE NULL END AS match_score
                FROM candidate_base cb
                WHERE cb.base_geom IS NOT NULL
            )
            SELECT 
                cc.candidate_id,
                cc.first_name,
                cc.last_name,
                cc.headline,
                cc.bio,
                cc.preferred_city,
                cc.job_search_status,
                cc.show_exact_location_to_employers,
                cc.commute_radius_meters,
                cc.display_longitude,
                cc.display_latitude,
                cc.distance_meters,
                cc.match_score,
                COALESCE((
                    SELECT ARRAY_AGG(s.name)
                    FROM candidate_skills cs
                    JOIN skills s ON s.id = cs.skill_id
                    WHERE cs.candidate_id = cc.candidate_id
                ), ARRAY[]::text[]) AS skills,
                (SELECT COUNT(*)::int4 FROM candidate_educations ce WHERE ce.candidate_id = cc.candidate_id) AS educations_count,
                COALESCE((
                    SELECT SUM(GREATEST(1, COALESCE(ce.end_year, 1403) - ce.start_year))::int4
                    FROM candidate_experiences ce
                    WHERE ce.candidate_id = cc.candidate_id
                ), 0) AS experience_years
            FROM candidate_computed cc
            WHERE (
                $5::float8 IS NULL OR
                (cc.display_longitude >= $5 AND cc.display_latitude >= $6 AND cc.display_longitude <= $7 AND cc.display_latitude <= $8)
            )
            AND (
                $11::float8 IS NULL OR
                (cc.distance_meters IS NOT NULL AND cc.distance_meters <= $11)
            )
            ORDER BY 
                cc.match_score DESC NULLS LAST,
                CASE WHEN cc.job_search_status = 'actively_looking' THEN 1 ELSE 2 END ASC,
                cc.distance_meters ASC NULLS LAST
            LIMIT $12
        "#;

        #[derive(sqlx::FromRow)]
        struct TalentDbRow {
            candidate_id: Uuid,
            first_name: String,
            last_name: String,
            headline: Option<String>,
            bio: Option<String>,
            preferred_city: Option<String>,
            job_search_status: String,
            show_exact_location_to_employers: bool,
            commute_radius_meters: i32,
            display_longitude: Option<f64>,
            display_latitude: Option<f64>,
            distance_meters: Option<f64>,
            match_score: Option<i16>,
            skills: Vec<String>,
            educations_count: i32,
            experience_years: i32,
        }

        let rows = sqlx::query_as::<_, TalentDbRow>(sql)
            .bind(q_text)
            .bind(actively_looking_only)
            .bind(city)
            .bind(skill_ids)
            .bind(west)
            .bind(south)
            .bind(east)
            .bind(north)
            .bind(c_lon)
            .bind(c_lat)
            .bind(radius_m)
            .bind(limit as i64)
            .bind(target_opp_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| {
            let coords = match (r.display_longitude, r.display_latitude) {
                (Some(lon), Some(lat)) => Some([lon, lat]),
                _ => None,
            };

            let mut match_reasons = Vec::new();
            if let Some(score) = r.match_score {
                if score >= 70 {
                    match_reasons.push("تطابق مهارت‌های تخصصی با نیازمندی موقعیت شغلی".to_string());
                }
                if let Some(dist) = r.distance_meters {
                    if dist <= r.commute_radius_meters as f64 {
                        match_reasons.push("سکونت در شعاع تردد مجاز تا شرکت".to_string());
                    }
                }
                if r.job_search_status == "actively_looking" {
                    match_reasons.push("جویای کار فوری و آماده مصاحبه".to_string());
                }
            }

            TalentSearchResult {
                candidate_id: r.candidate_id,
                first_name: r.first_name,
                last_name: r.last_name,
                headline: r.headline,
                bio: r.bio,
                preferred_city: r.preferred_city,
                job_search_status: r.job_search_status,
                skills: r.skills,
                coordinates: coords,
                distance_meters: r.distance_meters,
                match_score: r.match_score.map(|s| s.clamp(0, 100) as u8),
                match_reasons,
                has_exact_location: r.show_exact_location_to_employers,
                commute_radius_meters: r.commute_radius_meters,
                experience_years: r.experience_years,
                educations_count: r.educations_count as usize,
            }
        }).collect())
    }
}