use super::dto::{AuthResponseDto, LoginCommand, RegisterCommand};
use crate::error::ApplicationError;
use biz_domain::identity::{Email, NewUser, RawPassword};
use biz_storage::{PasswordService, StorageError, TokenRepository, TokenService, UserRepository};
use uuid::Uuid;




#[derive(Clone)]
pub struct AuthUseCases {
    user_repo: UserRepository,
    token_repo: TokenRepository,
    token_service: TokenService,
}

impl AuthUseCases {
    pub fn new(
        user_repo: UserRepository,
        token_repo: TokenRepository,
        token_service: TokenService,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            token_service,
        }
    }

    /// User Registration Workflow (Validate -> Hash Argon2id -> Insert -> Issue Tokens)
    pub async fn register(&self, cmd: RegisterCommand) -> Result<AuthResponseDto, ApplicationError> {
        let email = Email::new(&cmd.email)?;
        let raw_password = RawPassword::new(&cmd.password)?;

        let password_hash = PasswordService::hash_password(&raw_password)?;
        let new_user = NewUser {
            email: email.clone(),
            phone: cmd.phone,
            password_hash,
        };

        let user = self.user_repo.create_user(&new_user).await?;
        let tokens = self.token_service.issue_token_pair(user.id, email.as_str())?;

        let token_hash = TokenService::hash_token(&tokens.refresh_token);
        self.token_repo
            .save_refresh_token(user.id, &token_hash, self.token_service.refresh_token_expiry())
            .await?;

        Ok(AuthResponseDto {
            user_id: user.id,
            email: user.email,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in_secs: tokens.expires_in_secs,
        })
    }

    /// User Login Workflow (Verify Email & Password -> Issue Tokens & Rotate Refresh Token)
    pub async fn login(&self, cmd: LoginCommand) -> Result<AuthResponseDto, ApplicationError> {
        let (user, password_hash) = self
            .user_repo
            .find_by_email(&cmd.email)
            .await?
            .ok_or(StorageError::InvalidCredentials)?;

        if !PasswordService::verify_password(&cmd.password, &password_hash) {
            return Err(StorageError::InvalidCredentials.into());
        }

        let tokens = self.token_service.issue_token_pair(user.id, &user.email)?;

        let token_hash = TokenService::hash_token(&tokens.refresh_token);
        self.token_repo
            .save_refresh_token(user.id, &token_hash, self.token_service.refresh_token_expiry())
            .await?;

        Ok(AuthResponseDto {
            user_id: user.id,
            email: user.email,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in_secs: tokens.expires_in_secs,
        })
    }

    /// Refresh Token Rotation Workflow (Validate Hash -> Invalidate Old -> Issue New Pair)
    pub async fn refresh_tokens(&self, raw_refresh_token: &str) -> Result<AuthResponseDto, ApplicationError> {
        let old_token_hash = TokenService::hash_token(raw_refresh_token);

        let user_id = self
            .token_repo
            .find_active_user_id(&old_token_hash)
            .await?
            .ok_or_else(|| ApplicationError::Unauthorized("Invalid or expired refresh token".into()))?;

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        // Rotate: revoke old token immediately
        self.token_repo.revoke_token(&old_token_hash).await?;

        // Issue new token pair
        let tokens = self.token_service.issue_token_pair(user.id, &user.email)?;
        let new_token_hash = TokenService::hash_token(&tokens.refresh_token);
        self.token_repo
            .save_refresh_token(user.id, &new_token_hash, self.token_service.refresh_token_expiry())
            .await?;

        Ok(AuthResponseDto {
            user_id: user.id,
            email: user.email,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in_secs: tokens.expires_in_secs,
        })
    }

    /// Logout Workflow: Invalidates the specific refresh token
    pub async fn logout(&self, raw_refresh_token: &str) -> Result<(), ApplicationError> {
        let token_hash = TokenService::hash_token(raw_refresh_token);
        self.token_repo.revoke_token(&token_hash).await?;
        Ok(())
    }

    /// Verifies access token and extracts user ID
    pub fn authenticate(&self, access_token: &str) -> Result<Uuid, ApplicationError> {
        let claims = self.token_service.verify_access_token(access_token)?;
        Ok(claims.sub)
    }
}