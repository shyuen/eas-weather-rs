use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;

/// Startup check handler to verify if the server has started successfully
pub async fn startup() -> impl IntoResponse {
    Json(json!({"status": "ok", "message": "Server is started"}))
}

/// Readiness check handler to verify if the server is ready to accept requests
pub async fn readiness() -> impl IntoResponse {
    Json(json!({"status": "ok", "message": "Server is ready"}))
}

/// Liveness check handler to verify if the server is alive and healthy
pub async fn liveness() -> impl IntoResponse {
    Json(json!({"status": "ok", "message": "Server is running and healthy"}))
}
