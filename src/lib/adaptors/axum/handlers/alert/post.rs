use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::adaptors::axum::app_state::AppState;
use crate::adaptors::axum::handlers::alert::AlertSchema;
use crate::adaptors::axum::handlers::error::{ApiErrorResponse, JsonBody};
use crate::domain::alert::model::CreateAlertInput;
use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::ConfigPort;

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
pub(crate) async fn create_alert<C, AP>(
    State(state): State<AppState<C, AP>>,
    JsonBody(req): JsonBody<CreateAlertRequest>,
) -> impl IntoResponse
where
    C: ConfigPort,
    AP: AlertPort,
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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    use crate::adaptors::axum::handlers::alert::body_to_json;
    use crate::test_support::{
        DEFAULT_PAGE_LIMIT, FailingDb, MockDb, PAGE_LIMIT_MAX, build_alert_app, build_state,
        mock_config_service,
    };

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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
        let state = build_state::<FailingDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &FailingDb,
        );
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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
}
