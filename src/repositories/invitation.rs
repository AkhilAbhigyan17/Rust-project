use crate::models::Invitation;
use crate::utils::error::ApiResult;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool, org_id: Uuid, email: &str, role_id: Uuid, token_hash: &str,
    invited_by: Uuid, expires_at: DateTime<Utc>,
) -> ApiResult<Invitation> {
    let inv = sqlx::query_as::<_, Invitation>(
        r#"INSERT INTO invitations (organization_id, email, role_id, token_hash, invited_by, expires_at)
           VALUES ($1,$2,$3,$4,$5,$6) RETURNING *"#,
    )
    .bind(org_id).bind(email).bind(role_id).bind(token_hash).bind(invited_by).bind(expires_at)
    .fetch_one(pool).await?;
    Ok(inv)
}

pub async fn find_valid(pool: &PgPool, token_hash: &str) -> ApiResult<Option<Invitation>> {
    let inv = sqlx::query_as::<_, Invitation>(
        "SELECT * FROM invitations WHERE token_hash = $1 AND accepted_at IS NULL AND expires_at > NOW()",
    ).bind(token_hash).fetch_optional(pool).await?;
    Ok(inv)
}

pub async fn mark_accepted(pool: &PgPool, id: Uuid) -> ApiResult<()> {
    sqlx::query("UPDATE invitations SET accepted_at = NOW() WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(())
}
