//! Environment-driven configuration.
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_access_ttl_secs: i64,
    pub jwt_refresh_ttl_secs: i64,
    pub bind_addr: String,
    pub app_env: String,
    pub cors_allowed_origins: Option<String>,
}

impl Settings {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_access_ttl_secs: std::env::var("JWT_ACCESS_TTL_SECS")
                .unwrap_or_else(|_| "900".into())
                .parse()?,
            jwt_refresh_ttl_secs: std::env::var("JWT_REFRESH_TTL_SECS")
                .unwrap_or_else(|_| "1209600".into())
                .parse()?,
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            app_env: std::env::var("APP_ENV").unwrap_or_else(|_| "development".into()),
            cors_allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS").ok(),
        })
    }
}
