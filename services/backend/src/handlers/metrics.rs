use axum::response::IntoResponse;

pub async fn metrics_handler() -> impl IntoResponse {
    // Metrics are exposed on separate port via PrometheusBuilder
    "Metrics available on metrics port"
}
