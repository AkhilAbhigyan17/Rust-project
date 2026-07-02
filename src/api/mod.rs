//! Shared application state / dependency injection container.
use crate::config::Settings;
use crate::database::DbPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub settings: Arc<Settings>,
}

impl AppState {
    pub fn new(pool: DbPool, settings: Settings) -> Self {
        Self { pool, settings: Arc::new(settings) }
    }
}
