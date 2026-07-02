use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;
use validator::Validate;

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::dto::{CreateApiKeyRequest, CreateApiKeyResponse, MessageResponse};
use crate::permissions::require_permission;
use crate::repositories::api_key as repo;
use crate::services::api_key_service;
use crate::utils::error::{ApiError, ApiResult};

pub async fn list_keys(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(org_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&state.pool, claims.sub, org_id, "api_keys.manage").await?;
    let keys = repo::list_for_org(&state.pool, org_id).await?;
    Ok(Json(serde_json::json!({ "api_keys": keys })))
}

pub async fn create_key(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(org_id): Path<Uuid>,
    Json(body): Json<CreateApiKeyRequest>,
) -> ApiResult<Json<CreateApiKeyResponse>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    require_permission(&state.pool, claims.sub, org_id, "api_keys.manage").await?;
    let (key, full) = api_key_service::generate(&state.pool, org_id, claims.sub, &body.name, &body.scopes, body.expires_in_days).await?;
    Ok(Json(CreateApiKeyResponse { id: key.id, key: full, prefix: key.prefix, name: key.name }))
}

pub async fn rotate_key(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path((org_id, key_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<CreateApiKeyRequest>,
) -> ApiResult<Json<CreateApiKeyResponse>> {
    require_permission(&state.pool, claims.sub, org_id, "api_keys.manage").await?;
    let (key, full) = api_key_service::rotate(&state.pool, org_id, claims.sub, key_id, &body.name, &body.scopes, body.expires_in_days).await?;
    Ok(Json(CreateApiKeyResponse { id: key.id, key: full, prefix: key.prefix, name: key.name }))
}

pub async fn delete_key(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path((org_id, key_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<MessageResponse>> {
    require_permission(&state.pool, claims.sub, org_id, "api_keys.manage").await?;
    repo::revoke(&state.pool, key_id).await?;
    Ok(Json(MessageResponse { message: "api key revoked".into() }))
}
