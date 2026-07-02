//! IAM Platform - production-ready Identity & Access Management backend.
mod api;
mod auth;
mod config;
mod database;
mod domain;
mod dto;
mod handlers;
mod middleware;
mod models;
mod permissions;
mod repositories;
mod routes;
mod services;
mod utils;

use crate::config::Settings;
use crate::database::init_pool;
use crate::routes::build_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::from_env()?;
    tracing::info!(?settings.bind_addr, "starting iam-platform");

    let pool = init_pool(&settings.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = api::AppState::new(pool, settings.clone());
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&settings.bind_addr).await?;
    tracing::info!("listening on {}", settings.bind_addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}
