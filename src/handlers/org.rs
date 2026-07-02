use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;
use validator::Validate;

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::dto::*;
use crate::middleware::ClientInfo;
use crate::permissions::require_permission;
use crate::repositories::organization as org_repo;
use crate::services::{audit_service, org_service};
use crate::utils::error::{ApiError, ApiResult};

pub async fn list_my_orgs(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    let orgs = org_repo::list_for_user(&state.pool, claims.sub).await?;
    Ok(Json(serde_json::json!({ "organizations": orgs })))
}

pub async fn create_org(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    ClientInfo { ip, user_agent }: ClientInfo,
    Json(body): Json<CreateOrganizationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let org = org_service::create_organization(&state.pool, claims.sub, &body.name, &body.slug).await?;
    let _ = audit_service::log(&state.pool, Some(claims.sub), Some(org.id), "org.create",
        Some("organization"), Some(&org.id.to_string()), ip.as_deref(), user_agent.as_deref(),
        None, Some(serde_json::to_value(&org).unwrap_or_default())).await;
    Ok(Json(serde_json::json!({ "organization": org })))
}

pub async fn switch_org(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(org_id): Path<Uuid>,
) -> ApiResult<Json<AuthTokens>> {
    if !org_repo::is_member(&state.pool, org_id, claims.sub).await? {
        return Err(ApiError::Forbidden);
    }
    let user = crate::repositories::user::find_by_id(&state.pool, claims.sub).await?.ok_or(ApiError::NotFound)?;
    let access = crate::auth::issue_access_token(
        &state.settings.jwt_secret, user.id, &user.email, Some(org_id),
        state.settings.jwt_access_ttl_secs,
    )?;
    Ok(Json(AuthTokens {
        access_token: access,
        refresh_token: String::new(),
        token_type: "Bearer",
        expires_in: state.settings.jwt_access_ttl_secs,
    }))
}

pub async fn list_members(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(org_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&state.pool, claims.sub, org_id, "members.read").await?;
    let members = org_repo::list_members(&state.pool, org_id).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}

pub async fn invite_member(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    ClientInfo { ip, user_agent }: ClientInfo,
    Path(org_id): Path<Uuid>,
    Json(body): Json<InviteMemberRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    require_permission(&state.pool, claims.sub, org_id, "members.invite").await?;
    let (inv, token) = org_service::invite(&state.pool, org_id, &body.email, body.role_id, claims.sub).await?;
    let _ = audit_service::log(&state.pool, Some(claims.sub), Some(org_id), "member.invite",
        Some("invitation"), Some(&inv.id.to_string()), ip.as_deref(), user_agent.as_deref(),
        None, Some(serde_json::json!({"email": body.email, "role_id": body.role_id}))).await;
    Ok(Json(serde_json::json!({ "invitation_id": inv.id, "invite_token": token })))
}

pub async fn accept_invitation(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<AcceptInvitationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let org_id = org_service::accept_invitation(&state.pool, claims.sub, &body.token).await?;
    Ok(Json(serde_json::json!({ "organization_id": org_id })))
}

pub async fn remove_member(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    ClientInfo { ip, user_agent }: ClientInfo,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<MessageResponse>> {
    require_permission(&state.pool, claims.sub, org_id, "members.remove").await?;
    org_repo::remove_member(&state.pool, org_id, user_id).await?;
    let _ = audit_service::log(&state.pool, Some(claims.sub), Some(org_id), "member.remove",
        Some("user"), Some(&user_id.to_string()), ip.as_deref(), user_agent.as_deref(), None, None).await;
    Ok(Json(MessageResponse { message: "member removed".into() }))
}

pub async fn transfer_ownership(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    ClientInfo { ip, user_agent }: ClientInfo,
    Path(org_id): Path<Uuid>,
    Json(body): Json<TransferOwnershipRequest>,
) -> ApiResult<Json<MessageResponse>> {
    org_service::transfer_ownership(&state.pool, org_id, claims.sub, body.new_owner_user_id).await?;
    let _ = audit_service::log(&state.pool, Some(claims.sub), Some(org_id), "org.transfer_ownership",
        Some("organization"), Some(&org_id.to_string()), ip.as_deref(), user_agent.as_deref(),
        Some(serde_json::json!({"owner_id": claims.sub})),
        Some(serde_json::json!({"owner_id": body.new_owner_user_id}))).await;
    Ok(Json(MessageResponse { message: "ownership transferred".into() }))
}
