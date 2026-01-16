use axum::{extract::State, Json};
use redis::AsyncCommands;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn detailed_health(State(state): State<AppState>) -> Json<Value> {
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    let redis_healthy = {
        let mut conn = state.redis.clone();
        conn.get::<_, Option<String>>("_health_check")
            .await
            .is_ok()
    };

    Json(json!({
        "status": if db_healthy && redis_healthy { "healthy" } else { "degraded" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "database": if db_healthy { "healthy" } else { "unhealthy" },
            "redis": if redis_healthy { "healthy" } else { "unhealthy" },
        }
    }))
}
