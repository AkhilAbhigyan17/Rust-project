//! Permission evaluation engine.
use crate::database::DbPool;
use crate::utils::error::{ApiError, ApiResult};
use uuid::Uuid;

/// Returns the set of permission codes the user has within an organization.
pub async fn user_permissions_in_org(
    pool: &DbPool,
    user_id: Uuid,
    org_id: Uuid,
) -> ApiResult<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT p.code
        FROM organization_members m
        JOIN role_permissions rp ON rp.role_id = m.role_id
        JOIN permissions p ON p.id = rp.permission_id
        WHERE m.organization_id = $1 AND m.user_id = $2
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(c,)| c).collect())
}

pub async fn require_permission(
    pool: &DbPool,
    user_id: Uuid,
    org_id: Uuid,
    code: &str,
) -> ApiResult<()> {
    let perms = user_permissions_in_org(pool, user_id, org_id).await?;
    if perms.iter().any(|c| c == code) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}
