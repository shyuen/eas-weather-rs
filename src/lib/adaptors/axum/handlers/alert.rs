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
use utoipa::IntoParams;

use crate::adaptors::axum::openapi::AlertsListResponse;

#[derive(Deserialize, IntoParams)]
pub(crate) struct LatestAlertsParams {
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
}

/// Handler for GET /alerts
///
/// Returns the latest version of each alert, ordered by sent time descending,
/// with pagination.
#[utoipa::path(
    get,
    path = "/alerts",
    params(LatestAlertsParams),
    responses(
        (status = 200, description = "Latest alerts", body = AlertsListResponse),
        (status = 500, description = "Database error")
    ),
    tag = "alerts"
)]
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
        Ok(response) => {
            let body = AlertsListResponse {
                total: response.total,
                count: response.alerts.len() as u64,
                limit,
                offset,
                alerts: response.alerts,
            };
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

/// Handler for GET /alerts/daily
///
/// Returns the latest version of each alert sent within the last 24 hours,
/// fetched from the database via the alert port, with pagination.
#[utoipa::path(
    get,
    path = "/alerts/daily",
    params(LatestAlertsParams),
    responses(
        (status = 200, description = "Daily alerts", body = AlertsListResponse),
        (status = 500, description = "Database error")
    ),
    tag = "alerts"
)]
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
        Ok(response) => {
            let body = AlertsListResponse {
                total: response.total,
                count: response.alerts.len() as u64,
                limit,
                offset,
                alerts: response.alerts,
            };
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::test_support::{
        DEFAULT_PAGE_LIMIT, FailingDb, MockDb, MockMeta, PAGE_LIMIT_MAX, build_alert_app,
        build_state, build_webserver,
    };

    async fn body_to_json(response: axum::response::Response) -> serde_json::Value {
        let (_, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    // ── GET /alerts ──

    #[tokio::test]
    async fn test_get_alerts_default_params() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], DEFAULT_PAGE_LIMIT);
        assert_eq!(json["offset"], 0);
        assert_eq!(json["total"], 42);
        assert_eq!(json["count"], 0);
        assert!(json["alerts"].is_array());
    }

    #[tokio::test]
    async fn test_get_alerts_caps_limit() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?limit=100")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], PAGE_LIMIT_MAX);
    }

    #[tokio::test]
    async fn test_get_alerts_within_limit() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?limit=5")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], 5);
    }

    #[tokio::test]
    async fn test_get_alerts_with_offset() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?limit=5&offset=10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], 5);
        assert_eq!(json["offset"], 10);
    }

    #[tokio::test]
    async fn test_get_alerts_error() {
        let state = build_state::<FailingDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 500);
        let json = body_to_json(response).await;
        assert!(json["error"].is_string());
    }

    // ── GET /alerts/daily ──

    #[tokio::test]
    async fn test_get_daily_alerts_default_params() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/daily")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], DEFAULT_PAGE_LIMIT);
        assert_eq!(json["offset"], 0);
        assert_eq!(json["total"], 42);
    }

    #[tokio::test]
    async fn test_get_daily_alerts_caps_limit() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/daily?limit=100")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], PAGE_LIMIT_MAX);
    }

    #[tokio::test]
    async fn test_get_daily_alerts_error() {
        let state = build_state::<FailingDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/daily")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 500);
        let json = body_to_json(response).await;
        assert!(json["error"].is_string());
    }
}
