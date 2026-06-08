use crate::adaptors::axum::app_state::AppState;
use crate::domain::alert::port::AlertPort;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use tracing::error;

/// Handler for GET /alerts/daily
///
/// Returns the latest version of each alert sent within the last 24 hours,
/// fetched from the database via the alert port.
pub(crate) async fn get_daily_alerts<MR, DR>(
    State(state): State<AppState<MR, DR>>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    let alert_service = state.get_alert_service();

    match alert_service.get_daily_alerts().await {
        Ok(response) => (
            StatusCode::OK,
            Json(json!({
                "count": response.alerts.len(),
                "alerts": response.alerts,
            })),
        )
            .into_response(),
        Err(err) => {
            error!("failed to retrieve daily alerts: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response()
        }
    }
}
