use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::adaptors::axum::app_state::AppState;
use crate::adaptors::axum::handlers::alert::AlertSchema;
use crate::adaptors::axum::handlers::error::{ApiErrorResponse, ErrorCode, JsonBody};
use crate::domain::alert::model::UpdateAlertInput;
use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::ConfigPort;
use crate::domain::database::port::DatabasePort;

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
pub(crate) async fn update_alert<CP, AP, DP>(
    State(state): State<AppState<CP, AP, DP>>,
    Path(identifier): Path<String>,
    JsonBody(req): JsonBody<UpdateAlertRequest>,
) -> impl IntoResponse
where
    CP: ConfigPort,
    AP: AlertPort,
    DP: DatabasePort,
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
    use tower::ServiceExt;

    use crate::adaptors::axum::handlers::alert::body_to_json;
    use crate::test_support::{
        DEFAULT_PAGE_LIMIT, FailingDb, MockDb, PAGE_LIMIT_MAX, build_alert_app, build_state,
        mock_config_service,
    };

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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
        let state = build_state::<FailingDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &FailingDb,
        );
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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
