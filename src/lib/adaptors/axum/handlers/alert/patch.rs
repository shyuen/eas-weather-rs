use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::adaptors::axum::app_state::AppState;
use crate::adaptors::axum::handlers::alert::AlertSchema;
use crate::adaptors::axum::handlers::error::{ApiErrorResponse, ErrorCode, JsonBody};
use crate::domain::alert::model::PatchAlertInput;
use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::ConfigPort;
use crate::domain::database::port::DatabasePort;

/// Request body for partially updating an existing alert. The identifier comes
/// from the URL path. Omitted fields keep their existing values. Sending `null`
/// for a required field is a validation error; `null` for `source` or
/// `references` clears the field.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PatchAlertRequest {
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub sender: Option<Option<String>>,
    /// RFC 3339 timestamp, e.g. "2002-05-24T16:49:00-00:00".
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub sent: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub status: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub msg_type: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub source: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub scope: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub references: Option<Option<Vec<String>>>,
}

/// Deserialize an `Option<Option<T>>` so that an explicit JSON `null` yields
/// `Some(None)` (distinct from a missing field, which yields `None`). This lets
/// PATCH distinguish "clear this field" (`null`) from "leave it untouched"
/// (absent), and lets validation reject `null` on required fields.
fn deserialize_double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}

/// Convert the HTTP patch body into the domain's raw input contract.
impl From<PatchAlertRequest> for PatchAlertInput {
    fn from(req: PatchAlertRequest) -> Self {
        PatchAlertInput {
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

/// Handler for PATCH /alerts/{identifier}
///
/// Partially updates an existing alert, identified by the URL path. Omitted
/// fields keep their existing values; the path identifier is authoritative.
#[utoipa::path(
    patch,
    path = "/alerts/{identifier}",
    request_body = PatchAlertRequest,
    params(
        ("identifier" = String, Path, description = "Alert identifier")
    ),
    responses(
        (status = 200, description = "Alert updated", body = AlertSchema),
        (status = 400, description = "Validation error or invalid identifier", body = ApiErrorResponse),
        (status = 404, description = "Alert not found", body = ApiErrorResponse),
        (status = 422, description = "Malformed request body", body = ApiErrorResponse),
        (status = 500, description = "Database error", body = ApiErrorResponse)
    ),
    tag = "alerts"
)]
pub(crate) async fn patch_alert<CP, AP, DP>(
    State(state): State<AppState<CP, AP, DP>>,
    Path(identifier): Path<String>,
    JsonBody(req): JsonBody<PatchAlertRequest>,
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

    match alert_service.patch_alert(identifier, req.into()).await {
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

    #[tokio::test]
    async fn test_patch_alert_success() {
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
        let app = build_alert_app(state);
        let body = serde_json::json!({
            "sender": "PatchedSender",
            "status": "Test"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/alert-123")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["identifier"], "alert-123");
        assert_eq!(json["sender"], "PatchedSender");
        assert_eq!(json["status"], "Test");
        assert_eq!(json["msg_type"], "Alert");
    }

    #[tokio::test]
    async fn test_patch_alert_clear_source() {
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
        let app = build_alert_app(state);
        let body = serde_json::json!({
            "source": null
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/alert-123")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["source"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_patch_alert_null_required_field_rejected() {
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
        let app = build_alert_app(state);
        let body = serde_json::json!({
            "sender": null
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
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
        assert_eq!(json["message"], "field `sender` cannot be null");
    }

    #[tokio::test]
    async fn test_patch_alert_invalid_identifier() {
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
        let app = build_alert_app(state);
        let body = serde_json::json!({ "sender": "PatchedSender" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/Invalid%20Identifier")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    async fn test_patch_alert_validation_error() {
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
        let app = build_alert_app(state);
        let body = serde_json::json!({
            "sender": "Invalid Sender"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
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
    async fn test_patch_alert_db_error() {
        let state = build_state::<FailingDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &FailingDb,
        );
        let app = build_alert_app(state);
        let body = serde_json::json!({ "sender": "PatchedSender" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/alert-123")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 500);
    }

    #[tokio::test]
    async fn test_patch_alert_malformed_body_returns_422() {
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/alert-123")
                    .header("content-type", "application/json")
                    .body(Body::from("{not valid json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 422);
    }
}
