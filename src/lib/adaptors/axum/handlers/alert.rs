use crate::adaptors::axum::app_state::AppState;
use crate::adaptors::axum::handlers::error::{ApiErrorResponse, ErrorCode, JsonBody};
use crate::domain::alert::model::{Alert, CreateAlertInput, UpdateAlertInput};
use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::port::AlertPort;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams)]
pub(crate) struct LatestAlertsParams {
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
}

/// Paginated list of alerts returned by the alert endpoints.
///
/// `alerts` carries the domain `Alert` type directly (so the response is the
/// real serialized data), while the schema for that field is documented via
/// [`AlertSchema`] through utoipa's `value_type`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AlertsListResponse {
    pub total: u64,
    pub count: u64,
    pub limit: u64,
    pub offset: u64,
    #[schema(value_type = Vec<AlertSchema>)]
    pub alerts: Vec<Alert>,
}

/// Single alert, mirroring the serialized shape of `domain::alert::model::Alert`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AlertSchema {
    pub identifier: String,
    pub sender: String,
    pub sent: String,
    pub status: String,
    pub msg_type: String,
    pub source: Option<String>,
    pub scope: String,
    pub references: Vec<String>,
}

impl From<&Alert> for AlertSchema {
    fn from(alert: &Alert) -> Self {
        AlertSchema {
            identifier: alert.identifier().as_str().to_string(),
            sender: alert.sender().as_str().to_string(),
            sent: alert.sent().as_offset_date_time().to_string(),
            status: alert.status().as_str().to_string(),
            msg_type: alert.msg_type().as_str().to_string(),
            source: alert.source().as_opt_str().map(|s| s.to_string()),
            scope: alert.scope().as_str().to_string(),
            references: alert
                .references()
                .as_db_string()
                .map(|s| s.split(' ').map(|x| x.to_string()).collect())
                .unwrap_or_default(),
        }
    }
}

/// Request body for creating a new alert.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateAlertRequest {
    pub identifier: String,
    pub sender: String,
    /// RFC 3339 timestamp, e.g. "2002-05-24T16:49:00-00:00".
    pub sent: String,
    pub status: String,
    pub msg_type: String,
    pub source: Option<String>,
    pub scope: String,
    /// Each reference must be in the form `sender,identifier,sent`.
    pub references: Vec<String>,
}

/// Request body for replacing an existing alert. The identifier comes from the
/// URL path, not the body.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateAlertRequest {
    pub sender: String,
    /// RFC 3339 timestamp, e.g. "2002-05-24T16:49:00-00:00".
    pub sent: String,
    pub status: String,
    pub msg_type: String,
    pub source: Option<String>,
    pub scope: String,
    /// Each reference must be in the form `sender,identifier,sent`.
    pub references: Vec<String>,
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
        (status = 500, description = "Database error", body = ApiErrorResponse)
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
        Err(err) => ApiErrorResponse::from(err).into_response(),
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
        (status = 500, description = "Database error", body = ApiErrorResponse)
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
        Err(err) => ApiErrorResponse::from(err).into_response(),
    }
}

/// Convert the HTTP request body into the domain's raw input contract.
impl From<CreateAlertRequest> for CreateAlertInput {
    fn from(req: CreateAlertRequest) -> Self {
        CreateAlertInput {
            identifier: req.identifier,
            sender: req.sender,
            sent: req.sent,
            status: req.status,
            msg_type: req.msg_type,
            source: req.source,
            scope: req.scope,
            references: req.references,
        }
    }
}

/// Convert the HTTP update body into the domain's raw input contract.
impl From<UpdateAlertRequest> for UpdateAlertInput {
    fn from(req: UpdateAlertRequest) -> Self {
        UpdateAlertInput {
            sender: req.sender,
            sent: req.sent,
            status: req.status,
            msg_type: req.msg_type,
            source: req.source,
            scope: req.scope,
            references: req.references,
        }
    }
}

/// Handler for POST /alerts
///
/// Validates the request body into a domain `Alert` and persists it.
#[utoipa::path(
    post,
    path = "/alerts",
    request_body = CreateAlertRequest,
    responses(
        (status = 201, description = "Alert created", body = AlertSchema),
        (status = 400, description = "Validation error", body = ApiErrorResponse),
        (status = 422, description = "Malformed request body", body = ApiErrorResponse),
        (status = 500, description = "Database error", body = ApiErrorResponse)
    ),
    tag = "alerts"
)]
pub(crate) async fn create_alert<MR, DR>(
    State(state): State<AppState<MR, DR>>,
    JsonBody(req): JsonBody<CreateAlertRequest>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    let alert_service = state.get_alert_service();

    match alert_service.create_alert(req.into()).await {
        Ok(response) => (
            StatusCode::CREATED,
            Json(AlertSchema::from(&response.alert)),
        )
            .into_response(),
        Err(err) => ApiErrorResponse::from(err).into_response(),
    }
}

/// Handler for PUT /alerts/{identifier}
///
/// Replaces an existing alert, identified by the URL path. The path identifier
/// is authoritative; the body carries only the updateable fields.
#[utoipa::path(
    put,
    path = "/alerts/{identifier}",
    request_body = UpdateAlertRequest,
    params(
        ("identifier" = String, Path, description = "Alert identifier")
    ),
    responses(
        (status = 200, description = "Alert replaced", body = AlertSchema),
        (status = 400, description = "Validation error or invalid identifier", body = ApiErrorResponse),
        (status = 404, description = "Alert not found", body = ApiErrorResponse),
        (status = 422, description = "Malformed request body", body = ApiErrorResponse),
        (status = 500, description = "Database error", body = ApiErrorResponse)
    ),
    tag = "alerts"
)]
pub(crate) async fn update_alert<MR, DR>(
    State(state): State<AppState<MR, DR>>,
    Path(identifier): Path<String>,
    JsonBody(req): JsonBody<UpdateAlertRequest>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    let identifier = match AlertIdentifier::new(identifier) {
        Ok(id) => id,
        Err(err) => {
            return ApiErrorResponse::new(
                ErrorCode::InvalidIdentifier,
                err.to_string(),
                StatusCode::BAD_REQUEST,
            )
            .into_response();
        }
    };

    let alert_service = state.get_alert_service();

    match alert_service.update_alert(identifier, req.into()).await {
        Ok(response) => (StatusCode::OK, Json(AlertSchema::from(&response.alert))).into_response(),
        Err(err) => ApiErrorResponse::from(err).into_response(),
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
        assert_eq!(json["code"], "DATABASE_ERROR");
        assert!(json["message"].is_string());
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
        assert_eq!(json["code"], "DATABASE_ERROR");
        assert!(json["message"].is_string());
    }

    // ── POST /alerts ──

    fn valid_create_body() -> serde_json::Value {
        serde_json::json!({
            "identifier": "alert-123",
            "sender": "Sender123",
            "sent": "2002-05-24T16:49:00-00:00",
            "status": "Actual",
            "msg_type": "Alert",
            "source": "Weather Station 1",
            "scope": "Public",
            "references": ["Sender1,Alert123,2024-06-01T12:00:00-00:00"]
        })
    }

    #[tokio::test]
    async fn test_create_alert_success() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_body().to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 201);
        let json = body_to_json(response).await;
        assert_eq!(json["identifier"], "alert-123");
        assert_eq!(json["status"], "Actual");
    }

    #[tokio::test]
    async fn test_create_alert_validation_error() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let mut body = valid_create_body();
        body["sender"] = serde_json::json!("Invalid Sender");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);
        let json = body_to_json(response).await;
        assert_eq!(json["code"], "ALERT_VALIDATION_FAILED");
        assert!(json["message"].is_string());
    }

    #[tokio::test]
    async fn test_create_alert_bad_timestamp() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let mut body = valid_create_body();
        body["sent"] = serde_json::json!("not-a-timestamp");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    async fn test_create_alert_db_error() {
        let state = build_state::<FailingDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_body().to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 500);
    }

    #[tokio::test]
    async fn test_create_alert_missing_field_returns_422() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let mut body = valid_create_body();
        body.as_object_mut().unwrap().remove("identifier");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 422);
    }

    // ── PUT /alerts/{identifier} ──

    fn valid_update_body() -> serde_json::Value {
        serde_json::json!({
            "sender": "Sender456",
            "sent": "2003-01-01T12:00:00-00:00",
            "status": "Test",
            "msg_type": "Alert",
            "source": "Weather Station 2",
            "scope": "Public",
            "references": ["Sender2,Alert456,2024-06-02T12:00:00-00:00"]
        })
    }

    #[tokio::test]
    async fn test_update_alert_success() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/alert-123")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_update_body().to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["identifier"], "alert-123");
        assert_eq!(json["sender"], "Sender456");
        assert_eq!(json["status"], "Test");
    }

    #[tokio::test]
    async fn test_update_alert_invalid_identifier() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/Invalid%20Identifier")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_update_body().to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    async fn test_update_alert_validation_error() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let mut body = valid_update_body();
        body["sender"] = serde_json::json!("Invalid Sender");
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/alert-123")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);
        let json = body_to_json(response).await;
        assert_eq!(json["code"], "ALERT_VALIDATION_FAILED");
        assert!(json["message"].is_string());
    }

    #[tokio::test]
    async fn test_update_alert_db_error() {
        let state = build_state::<FailingDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/alert-123")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_update_body().to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 500);
    }

    #[tokio::test]
    async fn test_update_alert_missing_field_returns_422() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let mut body = valid_update_body();
        body.as_object_mut().unwrap().remove("sender");
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/alert-123")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 422);
    }
}
