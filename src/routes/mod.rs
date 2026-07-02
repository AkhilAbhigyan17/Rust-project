//! Route table wiring all handlers under `/api/v1`.
use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::AppState;
use crate::dto::*;
use crate::handlers::*;
use crate::middleware::rate_limit::{new_limiter, rate_limit};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::register, crate::handlers::auth::login, crate::handlers::auth::refresh,
        crate::handlers::auth::logout, crate::handlers::auth::verify_email, crate::handlers::auth::forgot_password,
        crate::handlers::auth::reset_password, crate::handlers::auth::change_password, crate::handlers::auth::me,
        crate::handlers::auth::list_sessions, crate::handlers::health::health,
    ),
    components(schemas(
        RegisterRequest, LoginRequest, AuthTokens, RefreshRequest,
        ForgotPasswordRequest, ResetPasswordRequest, ChangePasswordRequest,
        VerifyEmailRequest, MessageResponse, CreateOrganizationRequest,
        InviteMemberRequest, AcceptInvitationRequest, TransferOwnershipRequest,
        CreateRoleRequest, CreateApiKeyRequest, CreateApiKeyResponse,
    )),
    tags((name = "IAM", description = "Identity & Access Management"))
)]
pub struct ApiDoc;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let limiter = new_limiter();

    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh))
        .route("/logout", post(auth::logout))
        .route("/verify-email", post(auth::verify_email))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))
        .route("/change-password", post(auth::change_password))
        .route("/me", get(auth::me))
        .route("/sessions", get(auth::list_sessions));

    let org_routes = Router::new()
        .route("/", get(org::list_my_orgs).post(org::create_org))
        .route("/:org_id/switch", post(org::switch_org))
        .route("/:org_id/members", get(org::list_members))
        .route("/:org_id/invitations", post(org::invite_member))
        .route("/accept-invitation", post(org::accept_invitation))
        .route("/:org_id/members/:user_id", delete(org::remove_member))
        .route("/:org_id/transfer-ownership", post(org::transfer_ownership));

    let role_routes = Router::new()
        .route("/permissions", get(role::list_permissions))
        .route("/:org_id/roles", get(role::list_roles).post(role::create_role));

    let key_routes = Router::new()
        .route("/:org_id/api-keys", get(api_key::list_keys).post(api_key::create_key))
        .route("/:org_id/api-keys/:key_id", delete(api_key::delete_key))
        .route("/:org_id/api-keys/:key_id/rotate", post(api_key::rotate_key));

    let audit_routes = Router::new().route("/:org_id/audit-logs", get(audit::list_logs));

    let orgs = org_routes.merge(key_routes).merge(audit_routes);
    let api = Router::new()
        .nest("/auth", auth_routes)
        .nest("/organizations", orgs)
        .nest("/rbac", role_routes);

    Router::new()
        .route("/health", get(health::health))
        .nest("/api/v1", api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .layer(axum::middleware::from_fn(crate::middleware::request_id::add_request_id))
        .layer(axum::middleware::from_fn_with_state(limiter.clone(), rate_limit))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
