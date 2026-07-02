use crate::models::Session;
use crate::utils::error::ApiResult;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    refresh_token_hash: &str,
    user_agent: Option<&str>,
    ip: Option<&str>,
    expires_at: DateTime<Utc>,
    rotated_from: Option<Uuid>,
) -> ApiResult<Session> {
    let s = sqlx::query_as::<_, Session>(
        r#"INSERT INTO sessions (user_id, refresh_token_hash, user_agent, ip_address, expires_at, rotated_from)
           VALUES ($1,$2,$3,$4,$5,$6) RETURNING *"#,
    )
    .bind(user_id).bind(refresh_token_hash).bind(user_agent).bind(ip).bind(expires_at).bind(rotated_from)
    .fetch_one(pool).await?;
    Ok(s)
}

pub async fn find_active_by_hash(pool: &PgPool, hash: &str) -> ApiResult<Option<Session>> {
    let s = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE refresh_token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW()",
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?;
    Ok(s)
}

pub async fn revoke(pool: &PgPool, id: Uuid) -> ApiResult<()> {
    sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> ApiResult<()> {
    sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL")
        .bind(user_id).execute(pool).await?;
    Ok(())
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<Session>> {
    let list = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool).await?;
    Ok(list)
}
