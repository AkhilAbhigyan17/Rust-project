use crate::models::ApiKey;
use crate::utils::error::ApiResult;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool, org_id: Uuid, created_by: Uuid, name: &str, prefix: &str,
    key_hash: &str, scopes: &[String], expires_at: Option<DateTime<Utc>>,
) -> ApiResult<ApiKey> {
    let k = sqlx::query_as::<_, ApiKey>(
        r#"INSERT INTO api_keys (organization_id, created_by, name, prefix, key_hash, scopes, expires_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING *"#,
    )
    .bind(org_id).bind(created_by).bind(name).bind(prefix).bind(key_hash)
    .bind(scopes).bind(expires_at)
    .fetch_one(pool).await?;
    Ok(k)
}

pub async fn list_for_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<ApiKey>> {
    let list = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE organization_id = $1 ORDER BY created_at DESC",
    ).bind(org_id).fetch_all(pool).await?;
    Ok(list)
}

pub async fn revoke(pool: &PgPool, id: Uuid) -> ApiResult<()> {
    sqlx::query("UPDATE api_keys SET revoked_at = NOW() WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn find_active_by_hash(pool: &PgPool, hash: &str) -> ApiResult<Option<ApiKey>> {
    let k = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE key_hash = $1 AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > NOW())",
    ).bind(hash).fetch_optional(pool).await?;
    Ok(k)
}
