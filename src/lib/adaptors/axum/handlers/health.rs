use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::adaptors::axum::app_state::AppState;
use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::ConfigPort;
use crate::domain::database::port::DatabasePort;

/// Startup check handler to verify if the server has started successfully
/// We normally won't hit failure here because we ensure the init DB connection
/// works in main or application will exit.
pub(crate) async fn startup() -> impl IntoResponse {
    Json(json!({"status": "ok", "message": "Server is started"}))
}

/// Readiness check handler to verify if the server is ready to accept requests.
///
/// This performs an actual dependency check against the database connection to
/// confirm the service can serve traffic. If the database is unreachable, it
/// returns `503 Service Unavailable` so the orchestrator stops routing traffic
/// to this instance.
pub(crate) async fn readiness<CP, AP, DP>(State(state): State<AppState<CP, AP, DP>>) -> Response
where
    CP: ConfigPort,
    AP: AlertPort,
    DP: DatabasePort,
{
    let db_port = state.get_database_port();
    match db_port.check_health().await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({"status": "ok", "message": "Server is ready"})),
        )
            .into_response(),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "unavailable", "message": err.to_string()})),
        )
            .into_response(),
    }
}

/// Liveness check handler to verify if the server is alive and healthy
pub(crate) async fn liveness() -> impl IntoResponse {
    Json(json!({"status": "ok", "message": "Server is running and healthy"}))
}
