//! JWT issuance/validation and auth-context extractor.
use crate::api::AppState;
use crate::utils::error::ApiError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,           // user id
    pub email: String,
    pub org_id: Option<Uuid>,
    pub exp: usize,
    pub iat: usize,
    pub jti: Uuid,
}

pub fn issue_access_token(
    secret: &str,
    user_id: Uuid,
    email: &str,
    org_id: Option<Uuid>,
    ttl_secs: i64,
) -> Result<String, ApiError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        email: email.into(),
        org_id,
        iat: now.timestamp() as usize,
        exp: (now + Duration::seconds(ttl_secs)).timestamp() as usize,
        jti: Uuid::new_v4(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| ApiError::Internal)
}

pub fn decode_access_token(secret: &str, token: &str) -> Result<Claims, ApiError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| ApiError::Unauthorized)
}

/// Extractor placing decoded JWT claims into handler args.
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

#[async_trait::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;
        let claims = decode_access_token(&state.settings.jwt_secret, token)?;
        Ok(AuthUser(claims))
    }
}
