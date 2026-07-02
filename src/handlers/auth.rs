use axum::extract::State;
use axum::Json;
use validator::Validate;

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::dto::*;
use crate::middleware::ClientInfo;
use crate::services::{auth_service, audit_service};
use crate::utils::error::{ApiError, ApiResult};

#[utoipa::path(post, path = "/auth/register", request_body = RegisterRequest,
    responses((status = 201, body = MessageResponse)))]
pub async fn register(
    State(state): State<AppState>,
    ClientInfo { ip, user_agent }: ClientInfo,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<Json<MessageResponse>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let user = auth_service::register(&state.pool, &body.email, &body.password, body.display_name.as_deref()).await?;
    let _token = auth_service::issue_email_verification(&state.pool, user.id).await?;
    let _ = audit_service::log(&state.pool, Some(user.id), None, "user.register", Some("user"),
        Some(&user.id.to_string()), ip.as_deref(), user_agent.as_deref(), None, None).await;
    // In production, email `_token` via `lettre`. We return a message only.
    Ok(Json(MessageResponse { message: "registered; check email to verify".into() }))
}

#[utoipa::path(post, path = "/auth/login", request_body = LoginRequest,
    responses((status = 200, body = AuthTokens)))]
pub async fn login(
    State(state): State<AppState>,
    ClientInfo { ip, user_agent }: ClientInfo,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<AuthTokens>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let (user, tokens) = auth_service::login(&state.pool, &state.settings, &body.email, &body.password,
        user_agent.as_deref(), ip.as_deref()).await?;
    let _ = audit_service::log(&state.pool, Some(user.id), None, "user.login", Some("user"),
        Some(&user.id.to_string()), ip.as_deref(), user_agent.as_deref(), None, None).await;
    Ok(Json(tokens))
}

#[utoipa::path(post, path = "/auth/refresh", request_body = RefreshRequest,
    responses((status = 200, body = AuthTokens)))]
pub async fn refresh(
    State(state): State<AppState>,
    ClientInfo { ip, user_agent }: ClientInfo,
    Json(body): Json<RefreshRequest>,
) -> ApiResult<Json<AuthTokens>> {
    let tokens = auth_service::refresh(&state.pool, &state.settings, &body.refresh_token,
        user_agent.as_deref(), ip.as_deref()).await?;
    Ok(Json(tokens))
}

#[utoipa::path(post, path = "/auth/logout", request_body = RefreshRequest,
    responses((status = 200, body = MessageResponse)))]
pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> ApiResult<Json<MessageResponse>> {
    auth_service::logout(&state.pool, &body.refresh_token).await?;
    Ok(Json(MessageResponse { message: "logged out".into() }))
}

#[utoipa::path(post, path = "/auth/verify-email", request_body = VerifyEmailRequest,
    responses((status = 200, body = MessageResponse)))]
pub async fn verify_email(
    State(state): State<AppState>,
    Json(body): Json<VerifyEmailRequest>,
) -> ApiResult<Json<MessageResponse>> {
    auth_service::verify_email(&state.pool, &body.token).await?;
    Ok(Json(MessageResponse { message: "email verified".into() }))
}

#[utoipa::path(post, path = "/auth/forgot-password", request_body = ForgotPasswordRequest,
    responses((status = 200, body = MessageResponse)))]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordRequest>,
) -> ApiResult<Json<MessageResponse>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let _ = auth_service::issue_password_reset(&state.pool, &body.email).await?;
    // Always return generic message to avoid user enumeration.
    Ok(Json(MessageResponse { message: "if the email exists, a reset link was sent".into() }))
}

#[utoipa::path(post, path = "/auth/reset-password", request_body = ResetPasswordRequest,
    responses((status = 200, body = MessageResponse)))]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordRequest>,
) -> ApiResult<Json<MessageResponse>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    auth_service::reset_password(&state.pool, &body.token, &body.new_password).await?;
    Ok(Json(MessageResponse { message: "password reset".into() }))
}

#[utoipa::path(post, path = "/auth/change-password", request_body = ChangePasswordRequest,
    responses((status = 200, body = MessageResponse)))]
pub async fn change_password(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<ChangePasswordRequest>,
) -> ApiResult<Json<MessageResponse>> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    auth_service::change_password(&state.pool, claims.sub, &body.current_password, &body.new_password).await?;
    Ok(Json(MessageResponse { message: "password changed; other sessions revoked".into() }))
}

#[utoipa::path(get, path = "/auth/me", responses((status = 200)))]
pub async fn me(AuthUser(claims): AuthUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": claims.sub,
        "email": claims.email,
        "org_id": claims.org_id,
    }))
}

#[utoipa::path(get, path = "/auth/sessions", responses((status = 200)))]
pub async fn list_sessions(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    let sessions = crate::repositories::session::list_for_user(&state.pool, claims.sub).await?;
    let out: Vec<_> = sessions.into_iter().map(|s| serde_json::json!({
        "id": s.id, "user_agent": s.user_agent, "ip_address": s.ip_address,
        "created_at": s.created_at, "expires_at": s.expires_at, "revoked_at": s.revoked_at,
    })).collect();
    Ok(Json(serde_json::json!({ "sessions": out })))
}
