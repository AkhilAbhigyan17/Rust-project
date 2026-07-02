use crate::models::VerificationToken;
use crate::utils::error::ApiResult;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool, user_id: Uuid, kind: &str, token_hash: &str, expires_at: DateTime<Utc>,
) -> ApiResult<VerificationToken> {
    let v = sqlx::query_as::<_, VerificationToken>(
        r#"INSERT INTO verification_tokens (user_id, kind, token_hash, expires_at)
           VALUES ($1,$2,$3,$4) RETURNING *"#,
    )
    .bind(user_id).bind(kind).bind(token_hash).bind(expires_at)
    .fetch_one(pool).await?;
    Ok(v)
}

pub async fn find_valid(pool: &PgPool, kind: &str, token_hash: &str) -> ApiResult<Option<VerificationToken>> {
    let v = sqlx::query_as::<_, VerificationToken>(
        "SELECT * FROM verification_tokens WHERE kind = $1 AND token_hash = $2 AND used_at IS NULL AND expires_at > NOW()",
    )
    .bind(kind).bind(token_hash).fetch_optional(pool).await?;
    Ok(v)
}

pub async fn mark_used(pool: &PgPool, id: Uuid) -> ApiResult<()> {
    sqlx::query("UPDATE verification_tokens SET used_at = NOW() WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(())
}
