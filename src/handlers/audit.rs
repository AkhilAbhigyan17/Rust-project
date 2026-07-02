use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::permissions::require_permission;
use crate::repositories::audit as repo;
use crate::utils::error::ApiResult;

pub async fn list_logs(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(org_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&state.pool, claims.sub, org_id, "audit.read").await?;
    let logs = repo::list_for_org(&state.pool, org_id, 200).await?;
    Ok(Json(serde_json::json!({ "logs": logs })))
}
