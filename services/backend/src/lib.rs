// Library interface for backend - exposes modules for testing

pub mod config;
pub mod domain;
pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod repository;
pub mod services;
pub mod state;

use axum::{
    routing::{get, post},
    Router,
};
use state::AppState;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

/// Build the application router
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        .route("/health/detailed", get(handlers::health::detailed_health))
        // Bets
        .route("/api/bets", post(handlers::bets::create_bet))
        .route("/api/bets/:bet_id", get(handlers::bets::get_bet))
        .route("/api/bets", get(handlers::bets::list_user_bets))
        // External processor endpoints
        .route("/api/external/bets/pending", get(handlers::external::get_pending_bets))
        .route("/api/external/batches/:batch_id", post(handlers::external::update_batch))
        // Metrics
        .route("/metrics", get(handlers::metrics::metrics_handler))
        // State
        .with_state(state)
        // Middleware
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
}
