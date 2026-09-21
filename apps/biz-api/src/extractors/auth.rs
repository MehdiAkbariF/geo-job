use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use uuid::Uuid;

pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".into()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized("Invalid authorization format. Use Bearer token".into()));
        }

        let token = &auth_header[7..];
        let claims = app_state
            .token_service
            .verify_access_token(token)
            .map_err(|_| ApiError::Unauthorized("Invalid or expired access token".into()))?;

        Ok(AuthenticatedUser { user_id: claims.sub })
    }
}

/// Optional authenticator: extracts user_id if Bearer token present and valid, returns None otherwise without failing
pub struct MaybeAuthenticatedUser(pub Option<Uuid>);

#[async_trait]
impl<S> FromRequestParts<S> for MaybeAuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        match auth_header {
            Some(header_val) if header_val.starts_with("Bearer ") => {
                let token = &header_val[7..];
                match app_state.token_service.verify_access_token(token) {
                    Ok(claims) => Ok(MaybeAuthenticatedUser(Some(claims.sub))),
                    Err(_) => Ok(MaybeAuthenticatedUser(None)),
                }
            }
            _ => Ok(MaybeAuthenticatedUser(None)),
        }
    }
}