use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::api::AppState;

/// Liveness / readiness probe.
#[utoipa::path(get, path = "/health", responses((status = 200)))]
pub async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    Json(json!({ "status": if db_ok { "ok" } else { "degraded" }, "database": db_ok }))
}
