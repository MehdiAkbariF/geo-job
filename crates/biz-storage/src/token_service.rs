use crate::error::StorageError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in_secs: u64,
}

#[derive(Clone)]
pub struct TokenService {
    jwt_secret: String,
    access_token_ttl_mins: i64,
    refresh_token_ttl_days: i64,
}

impl TokenService {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            access_token_ttl_mins: 15,
            refresh_token_ttl_days: 7,
        }
    }

    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn generate_refresh_token() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn issue_token_pair(&self, user_id: Uuid, email: &str) -> Result<TokenPair, StorageError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.access_token_ttl_mins);

        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| StorageError::Security(e.to_string()))?;

        let refresh_token = Self::generate_refresh_token();

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in_secs: (self.access_token_ttl_mins * 60) as u64,
        })
    }

    pub fn verify_access_token(&self, token: &str) -> Result<Claims, StorageError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| StorageError::InvalidCredentials)
    }

    pub fn refresh_token_expiry(&self) -> chrono::DateTime<Utc> {
        Utc::now() + Duration::days(self.refresh_token_ttl_days)
    }
}