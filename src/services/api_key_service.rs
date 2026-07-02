use crate::database::DbPool;
use crate::models::ApiKey;
use crate::repositories::api_key as repo;
use crate::utils::error::ApiResult;
use crate::utils::token::{hash_token, random_token};
use chrono::{Duration, Utc};
use uuid::Uuid;

pub async fn generate(
    pool: &DbPool, org_id: Uuid, user_id: Uuid, name: &str, scopes: &[String], expires_in_days: Option<i64>,
) -> ApiResult<(ApiKey, String)> {
    let secret = random_token(32);
    let prefix: String = secret.chars().take(8).collect();
    let full = format!("iam_{}_{}", prefix, secret);
    let hash = hash_token(&full);
    let expires_at = expires_in_days.map(|d| Utc::now() + Duration::days(d));
    let key = repo::create(pool, org_id, user_id, name, &prefix, &hash, scopes, expires_at).await?;
    Ok((key, full))
}

pub async fn rotate(pool: &DbPool, org_id: Uuid, user_id: Uuid, key_id: Uuid, name: &str, scopes: &[String], expires_in_days: Option<i64>) -> ApiResult<(ApiKey, String)> {
    repo::revoke(pool, key_id).await?;
    generate(pool, org_id, user_id, name, scopes, expires_in_days).await
}
