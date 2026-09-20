use crate::error::StorageError;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct TokenRepository {
    pool: PgPool,
}

impl TokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let sql = r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
        "#;

        sqlx::query(sql)
            .bind(user_id)
            .bind(token_hash)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Finds active, non-revoked, and non-expired token by its hash.
    pub async fn find_active_user_id(&self, token_hash: &str) -> Result<Option<Uuid>, StorageError> {
        let sql = r#"
            SELECT user_id
            FROM refresh_tokens
            WHERE token_hash = $1
              AND revoked_at IS NULL
              AND expires_at > NOW()
        "#;

        let user_id: Option<Uuid> = sqlx::query_scalar(sql)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user_id)
    }

    /// Revokes a specific refresh token (used during Logout and Token Rotation).
    pub async fn revoke_token(&self, token_hash: &str) -> Result<(), StorageError> {
        let sql = r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE token_hash = $1
        "#;

        sqlx::query(sql)
            .bind(token_hash)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Revokes all active refresh tokens for a user (security breach or password change).
    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), StorageError> {
        let sql = r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE user_id = $1 AND revoked_at IS NULL
        "#;

        sqlx::query(sql)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}