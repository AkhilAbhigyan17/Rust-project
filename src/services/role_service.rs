use crate::database::DbPool;
use crate::models::Role;
use crate::repositories::role as role_repo;
use crate::utils::error::ApiResult;
use uuid::Uuid;

pub async fn create_custom_role(
    pool: &DbPool, org_id: Uuid, name: &str, description: Option<&str>, perms: &[String],
) -> ApiResult<Role> {
    let role = role_repo::create(pool, Some(org_id), name, description, false).await?;
    role_repo::attach_permissions_by_code(pool, role.id, perms).await?;
    Ok(role)
}
