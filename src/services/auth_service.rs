use crate::auth::issue_access_token;
use crate::config::Settings;
use crate::database::DbPool;
use crate::dto::AuthTokens;
use crate::models::User;
use crate::repositories::{session as session_repo, user as user_repo, verification as ver_repo};
use crate::utils::error::{ApiError, ApiResult};
use crate::utils::password::{hash_password, validate_password_policy, verify_password};
use crate::utils::token::{hash_token, random_token};
use chrono::{Duration, Utc};
use uuid::Uuid;

const REFRESH_BYTES: usize = 48;

pub async fn register(pool: &DbPool, email: &str, password: &str, display_name: Option<&str>) -> ApiResult<User> {
    validate_password_policy(password)?;
    if user_repo::find_by_email(pool, email).await?.is_some() {
        return Err(ApiError::Conflict("email already registered".into()));
    }
    let hash = hash_password(password)?;
    user_repo::create(pool, email, &hash, display_name).await
}

pub async fn issue_tokens(
    pool: &DbPool, settings: &Settings, user: &User, org_id: Option<Uuid>,
    ua: Option<&str>, ip: Option<&str>,
) -> ApiResult<AuthTokens> {
    let access = issue_access_token(&settings.jwt_secret, user.id, &user.email, org_id, settings.jwt_access_ttl_secs)?;
    let refresh_plain = random_token(REFRESH_BYTES);
    let refresh_hash = hash_token(&refresh_plain);
    let exp = Utc::now() + Duration::seconds(settings.jwt_refresh_ttl_secs);
    session_repo::create(pool, user.id, &refresh_hash, ua, ip, exp, None).await?;
    Ok(AuthTokens {
        access_token: access,
        refresh_token: refresh_plain,
        token_type: "Bearer",
        expires_in: settings.jwt_access_ttl_secs,
    })
}

pub async fn login(pool: &DbPool, settings: &Settings, email: &str, password: &str, ua: Option<&str>, ip: Option<&str>) -> ApiResult<(User, AuthTokens)> {
    let user = user_repo::find_by_email(pool, email).await?.ok_or(ApiError::Unauthorized)?;
    if !user.is_active {
        return Err(ApiError::Forbidden);
    }
    if !verify_password(password, &user.password_hash) {
        return Err(ApiError::Unauthorized);
    }
    let tokens = issue_tokens(pool, settings, &user, None, ua, ip).await?;
    Ok((user, tokens))
}

pub async fn refresh(pool: &DbPool, settings: &Settings, refresh_token: &str, ua: Option<&str>, ip: Option<&str>) -> ApiResult<AuthTokens> {
    let hash = hash_token(refresh_token);
    let old = session_repo::find_active_by_hash(pool, &hash).await?.ok_or(ApiError::Unauthorized)?;
    // rotation: revoke old, mint new
    session_repo::revoke(pool, old.id).await?;
    let user = user_repo::find_by_id(pool, old.user_id).await?.ok_or(ApiError::Unauthorized)?;

    let access = issue_access_token(&settings.jwt_secret, user.id, &user.email, None, settings.jwt_access_ttl_secs)?;
    let new_refresh = random_token(REFRESH_BYTES);
    let new_hash = hash_token(&new_refresh);
    let exp = Utc::now() + Duration::seconds(settings.jwt_refresh_ttl_secs);
    session_repo::create(pool, user.id, &new_hash, ua, ip, exp, Some(old.id)).await?;
    Ok(AuthTokens {
        access_token: access,
        refresh_token: new_refresh,
        token_type: "Bearer",
        expires_in: settings.jwt_access_ttl_secs,
    })
}

pub async fn logout(pool: &DbPool, refresh_token: &str) -> ApiResult<()> {
    let hash = hash_token(refresh_token);
    if let Some(s) = session_repo::find_active_by_hash(pool, &hash).await? {
        session_repo::revoke(pool, s.id).await?;
    }
    Ok(())
}

pub async fn change_password(pool: &DbPool, user_id: Uuid, current: &str, new: &str) -> ApiResult<()> {
    validate_password_policy(new)?;
    let user = user_repo::find_by_id(pool, user_id).await?.ok_or(ApiError::NotFound)?;
    if !verify_password(current, &user.password_hash) {
        return Err(ApiError::Unauthorized);
    }
    let hash = hash_password(new)?;
    user_repo::update_password(pool, user_id, &hash).await?;
    session_repo::revoke_all_for_user(pool, user_id).await?;
    Ok(())
}

pub async fn issue_email_verification(pool: &DbPool, user_id: Uuid) -> ApiResult<String> {
    let plain = random_token(32);
    let hash = hash_token(&plain);
    let exp = Utc::now() + Duration::hours(24);
    ver_repo::create(pool, user_id, "email_verify", &hash, exp).await?;
    Ok(plain)
}

pub async fn verify_email(pool: &DbPool, token: &str) -> ApiResult<()> {
    let hash = hash_token(token);
    let v = ver_repo::find_valid(pool, "email_verify", &hash).await?.ok_or(ApiError::BadRequest("invalid or expired token".into()))?;
    user_repo::mark_email_verified(pool, v.user_id).await?;
    ver_repo::mark_used(pool, v.id).await?;
    Ok(())
}

pub async fn issue_password_reset(pool: &DbPool, email: &str) -> ApiResult<Option<String>> {
    if let Some(user) = user_repo::find_by_email(pool, email).await? {
        let plain = random_token(32);
        let hash = hash_token(&plain);
        let exp = Utc::now() + Duration::hours(1);
        ver_repo::create(pool, user.id, "password_reset", &hash, exp).await?;
        return Ok(Some(plain));
    }
    Ok(None)
}

pub async fn reset_password(pool: &DbPool, token: &str, new_password: &str) -> ApiResult<()> {
    validate_password_policy(new_password)?;
    let hash = hash_token(token);
    let v = ver_repo::find_valid(pool, "password_reset", &hash).await?
        .ok_or(ApiError::BadRequest("invalid or expired token".into()))?;
    let ph = hash_password(new_password)?;
    user_repo::update_password(pool, v.user_id, &ph).await?;
    ver_repo::mark_used(pool, v.id).await?;
    session_repo::revoke_all_for_user(pool, v.user_id).await?;
    Ok(())
}
