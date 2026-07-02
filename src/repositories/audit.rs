use crate::models::AuditLog;
use crate::utils::error::ApiResult;
use sqlx::PgPool;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn insert(
    pool: &PgPool,
    user_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    action: &str,
    resource_type: Option<&str>,
    resource_id: Option<&str>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    previous_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
) -> ApiResult<()> {
    sqlx::query(
        r#"INSERT INTO audit_logs
           (user_id, organization_id, action, resource_type, resource_id,
            ip_address, user_agent, previous_value, new_value)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
    )
    .bind(user_id).bind(organization_id).bind(action)
    .bind(resource_type).bind(resource_id)
    .bind(ip_address).bind(user_agent)
    .bind(previous_value).bind(new_value)
    .execute(pool).await?;
    Ok(())
}

pub async fn list_for_org(pool: &PgPool, org_id: Uuid, limit: i64) -> ApiResult<Vec<AuditLog>> {
    let list = sqlx::query_as::<_, AuditLog>(
        "SELECT * FROM audit_logs WHERE organization_id = $1 ORDER BY created_at DESC LIMIT $2",
    ).bind(org_id).bind(limit).fetch_all(pool).await?;
    Ok(list)
}
