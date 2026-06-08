use crate::adaptors::axum::app_state::AppState;
use crate::domain::alert::port::AlertPort;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;
use tracing::error;

#[derive(Deserialize)]
pub(crate) struct LatestAlertsParams {
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
}

/// Handler for GET /alerts
///
/// Returns the latest version of each alert, ordered by sent time descending,
/// with pagination.
pub(crate) async fn get_alerts<MR, DR>(
    State(state): State<AppState<MR, DR>>,
    Query(params): Query<LatestAlertsParams>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    let conf = state.get_meta_port().get_conf();
    let ws_conf = conf.get_webserver_config();
    let default_limit = ws_conf.default_page_limit.get();
    let page_limit_max = ws_conf.page_limit_max.get();
    let limit = params.limit.unwrap_or(default_limit).min(page_limit_max);
    let offset = params.offset.unwrap_or(0);

    let alert_service = state.get_alert_service();

    match alert_service.get_latest_alerts(limit, offset).await {
        Ok(response) => (
            StatusCode::OK,
            Json(json!({
                "total": response.total,
                "count": response.alerts.len(),
                "limit": limit,
                "offset": offset,
                "alerts": response.alerts,
            })),
        )
            .into_response(),
        Err(err) => {
            error!("failed to retrieve latest alerts: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response()
        }
    }
}

/// Handler for GET /alerts/daily
///
/// Returns the latest version of each alert sent within the last 24 hours,
/// fetched from the database via the alert port, with pagination.
pub(crate) async fn get_daily_alerts<MR, DR>(
    State(state): State<AppState<MR, DR>>,
    Query(params): Query<LatestAlertsParams>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    let conf = state.get_meta_port().get_conf();
    let ws_conf = conf.get_webserver_config();
    let default_limit = ws_conf.default_page_limit.get();
    let page_limit_max = ws_conf.page_limit_max.get();
    let limit = params.limit.unwrap_or(default_limit).min(page_limit_max);
    let offset = params.offset.unwrap_or(0);

    let alert_service = state.get_alert_service();

    match alert_service.get_daily_alerts(limit, offset).await {
        Ok(response) => (
            StatusCode::OK,
            Json(json!({
                "total": response.total,
                "count": response.alerts.len(),
                "limit": limit,
                "offset": offset,
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
