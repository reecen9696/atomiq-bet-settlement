use crate::config::Config;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub redis: ConnectionManager,
}

impl AppState {
    pub fn new(config: Config, db: PgPool, redis: ConnectionManager) -> Self {
        Self {
            config: Arc::new(config),
            db,
            redis,
        }
    }
}
