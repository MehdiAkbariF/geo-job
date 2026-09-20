use crate::error::StorageError;
use biz_domain::identity::{NewUser, User, UserStatus};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct UserDbRow {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub password_hash: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserDbRow {
    fn to_domain(self) -> User {
        let status = match self.status.as_str() {
            "suspended" => UserStatus::Suspended,
            "pending_verification" => UserStatus::PendingVerification,
            _ => UserStatus::Active,
        };

        User {
            id: self.id,
            email: self.email,
            phone: self.phone,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, new_user: &NewUser) -> Result<User, StorageError> {
        let sql = r#"
            INSERT INTO users (email, phone, password_hash, status)
            VALUES ($1, $2, $3, 'active')
            RETURNING id, email, phone, password_hash, status, created_at, updated_at
        "#;

        let res = sqlx::query_as::<_, UserDbRow>(sql)
            .bind(new_user.email.as_str())
            .bind(&new_user.phone)
            .bind(&new_user.password_hash)
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(row) => Ok(row.to_domain()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                let constraint = db_err.constraint().unwrap_or("");
                if constraint.contains("email") {
                    Err(StorageError::EmailAlreadyExists)
                } else if constraint.contains("phone") {
                    Err(StorageError::PhoneAlreadyExists)
                } else {
                    Err(StorageError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(StorageError::Database(e)),
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<(User, String)>, StorageError> {
        let sql = r#"
            SELECT id, email, phone, password_hash, status, created_at, updated_at
            FROM users
            WHERE LOWER(email) = LOWER($1)
        "#;

        let row: Option<UserDbRow> = sqlx::query_as(sql)
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| {
            let hash = r.password_hash.clone();
            (r.to_domain(), hash)
        }))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, StorageError> {
        let sql = r#"
            SELECT id, email, phone, password_hash, status, created_at, updated_at
            FROM users
            WHERE id = $1
        "#;

        let row: Option<UserDbRow> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(UserDbRow::to_domain))
    }
}