use crate::error::StorageError;
use biz_domain::identity::{User, UserStatus};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct UserDbRow {
    id: Uuid,
    email: Option<String>,
    phone: Option<String>,
    password_hash: Option<String>,
    user_type: String,
    national_id: Option<String>,
    is_phone_verified: bool,
    is_onboarded: bool,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl UserDbRow {
    fn to_domain(self) -> Result<User, StorageError> {
        let status = UserStatus::from_str(&self.status)
            .ok_or_else(|| StorageError::Security("وضعیت کاربر نامعتبر است".into()))?;

        Ok(User {
            id: self.id,
            email: self.email,
            phone: self.phone,
            password_hash: self.password_hash,
            user_type: self.user_type,
            national_id: self.national_id,
            is_phone_verified: self.is_phone_verified,
            is_onboarded: self.is_onboarded,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct OtpDbRow {
    pub id: Uuid,
    pub phone: String,
    pub code_hash: String,
    pub attempts: i32,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, email: Option<&str>, phone: Option<&str>, password_hash: Option<&str>) -> Result<User, StorageError> {
        let sql = r#"
            INSERT INTO users (email, phone, password_hash, status, user_type, is_phone_verified, is_onboarded)
            VALUES ($1, $2, $3, 'active', 'candidate', false, false)
            RETURNING id, email, phone, password_hash, user_type, national_id, is_phone_verified, is_onboarded, status, created_at, updated_at
        "#;

        let row = sqlx::query_as::<_, UserDbRow>(sql)
            .bind(email)
            .bind(phone)
            .bind(password_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(ref db_err) = e {
                    if db_err.constraint() == Some("users_email_key") {
                        return StorageError::EmailAlreadyExists;
                    }
                    if db_err.constraint() == Some("users_phone_key") {
                        return StorageError::PhoneAlreadyExists;
                    }
                }
                StorageError::Database(e)
            })?;

        row.to_domain()
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, StorageError> {
        let sql = "SELECT * FROM users WHERE id = $1";
        let row: Option<UserDbRow> = sqlx::query_as(sql).bind(id).fetch_optional(&self.pool).await?;
        row.map(|r| r.to_domain()).transpose()
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, StorageError> {
        let sql = "SELECT * FROM users WHERE LOWER(email) = LOWER($1)";
        let row: Option<UserDbRow> = sqlx::query_as(sql).bind(email).fetch_optional(&self.pool).await?;
        row.map(|r| r.to_domain()).transpose()
    }

    pub async fn find_by_phone(&self, phone: &str) -> Result<Option<User>, StorageError> {
        let sql = "SELECT * FROM users WHERE phone = $1";
        let row: Option<UserDbRow> = sqlx::query_as(sql).bind(phone).fetch_optional(&self.pool).await?;
        row.map(|r| r.to_domain()).transpose()
    }

    /// ثبت یا بازیابی کاربر با شماره موبایل (لاگین پیامکی بدون پسورد)
    pub async fn find_or_create_by_phone(&self, phone: &str) -> Result<(User, bool), StorageError> {
        if let Some(user) = self.find_by_phone(phone).await? {
            return Ok((user, false));
        }

        let sql = r#"
            INSERT INTO users (phone, status, user_type, is_phone_verified, is_onboarded)
            VALUES ($1, 'active', 'candidate', true, false)
            RETURNING id, email, phone, password_hash, user_type, national_id, is_phone_verified, is_onboarded, status, created_at, updated_at
        "#;

        let row = sqlx::query_as::<_, UserDbRow>(sql)
            .bind(phone)
            .fetch_one(&self.pool)
            .await?;

        Ok((row.to_domain()?, true))
    }

    // ==========================================
    // متدهای مدیریت کدهای پیامکی OTP
    // ==========================================

    pub async fn create_phone_otp(&self, phone: &str, code_hash: &str, expires_at: DateTime<Utc>) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO phone_verifications (phone, code_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id
        "#;

        let id: Uuid = sqlx::query_scalar(sql)
            .bind(phone)
            .bind(code_hash)
            .bind(expires_at)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }

    pub async fn get_latest_valid_otp(&self, phone: &str) -> Result<Option<OtpDbRow>, StorageError> {
        let sql = r#"
            SELECT id, phone, code_hash, attempts, expires_at, verified_at
            FROM phone_verifications
            WHERE phone = $1 AND verified_at IS NULL AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
        "#;

        let row = sqlx::query_as::<_, OtpDbRow>(sql)
            .bind(phone)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    pub async fn increment_otp_attempts(&self, otp_id: Uuid) -> Result<(), StorageError> {
        sqlx::query("UPDATE phone_verifications SET attempts = attempts + 1 WHERE id = $1")
            .bind(otp_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn mark_otp_verified(&self, otp_id: Uuid) -> Result<(), StorageError> {
        sqlx::query("UPDATE phone_verifications SET verified_at = NOW() WHERE id = $1")
            .bind(otp_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_onboarding_completed(&self, user_id: Uuid, user_type: &str, national_id: Option<&str>) -> Result<(), StorageError> {
        let sql = r#"
            UPDATE users
            SET user_type = $2,
                national_id = COALESCE($3, national_id),
                is_onboarded = true,
                updated_at = NOW()
            WHERE id = $1
        "#;

        sqlx::query(sql)
            .bind(user_id)
            .bind(user_type)
            .bind(national_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}