use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;
use validator::Validate;

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::dto::CreateRoleRequest;
use crate::permissions::require_permission;
use crate::repositories::role as role_repo;
use crate::services::role_service;
use crate::utils::error::{ApiError, ApiResult};

pub async fn list_permissions(
    State(state): State<AppState>,
    AuthUser(_): AuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    let perms = role_repo::list_permissions(&state.pool).await?;
    Ok(Json(serde_json::json!({ "permissions": perms })))
}

pub async fn list_roles(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(org_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&state.pool, claims.sub, org_id, "org.read").await?;
    let roles = role_repo::list_for_org(&state.pool, org_id).await?;
    Ok(Json(serde_json::json!({ "roles": roles })))
}

pub async fn create_role(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(org_id): Path<Uuid>,
    Json(body): Json<CreateRoleRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    require_permission(&state.pool, claims.sub, org_id, "roles.manage").await?;
    let role = role_service::create_custom_role(
        &state.pool, org_id, &body.name, body.description.as_deref(), &body.permission_codes,
    ).await?;
    Ok(Json(serde_json::json!({ "role": role })))
}
