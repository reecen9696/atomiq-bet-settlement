use crate::config::Config;
use redis::aio::ConnectionManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub redis: ConnectionManager,
}

impl AppState {
    pub fn new(config: Config, redis: ConnectionManager) -> Self {
        Self {
            config: Arc::new(config),
            redis,
        }
    }
}
