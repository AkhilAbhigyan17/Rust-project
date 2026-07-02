use crate::database::DbPool;
use crate::repositories::audit as repo;
use crate::utils::error::ApiResult;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn log(
    pool: &DbPool,
    user_id: Option<Uuid>, org_id: Option<Uuid>,
    action: &str, resource_type: Option<&str>, resource_id: Option<&str>,
    ip: Option<&str>, ua: Option<&str>,
    prev: Option<serde_json::Value>, new: Option<serde_json::Value>,
) -> ApiResult<()> {
    repo::insert(pool, user_id, org_id, action, resource_type, resource_id, ip, ua, prev, new).await
}
