use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::adaptors::axum::app_state::AppState;
use crate::adaptors::axum::handlers::error::{ApiErrorResponse, ErrorCode};
use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::port::AlertPort;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;

/// Handler for DELETE /alerts/{identifier}
///
/// Deletes an existing alert, identified by the URL path. The path identifier
/// is authoritative; no request body is expected.
#[utoipa::path(
    delete,
    path = "/alerts/{identifier}",
    params(
        ("identifier" = String, Path, description = "Alert identifier")
    ),
    responses(
        (status = 204, description = "Alert deleted"),
        (status = 400, description = "Invalid identifier", body = ApiErrorResponse),
        (status = 404, description = "Alert not found", body = ApiErrorResponse),
        (status = 500, description = "Database error", body = ApiErrorResponse)
    ),
    tag = "alerts"
)]
pub(crate) async fn delete_alert<MR, DR>(
    State(state): State<AppState<MR, DR>>,
    Path(identifier): Path<String>,
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

    match alert_service.delete_alert(identifier).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => ApiErrorResponse::from(err).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::adaptors::axum::handlers::alert::body_to_json;
    use crate::test_support::{
        DEFAULT_PAGE_LIMIT, FailingDb, MissingDb, MockDb, MockMeta, PAGE_LIMIT_MAX,
        build_alert_app, build_state, build_webserver,
    };

    #[tokio::test]
    async fn test_delete_alert_success() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/alert-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 204);
        let (_, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        assert!(bytes.is_empty());
    }

    #[tokio::test]
    async fn test_delete_alert_invalid_identifier() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/Invalid%20Identifier")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);
        let json = body_to_json(response).await;
        assert_eq!(json["code"], "INVALID_IDENTIFIER");
    }

    #[tokio::test]
    async fn test_delete_alert_not_found() {
        let state = build_state::<MissingDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/alert-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 404);
        let json = body_to_json(response).await;
        assert_eq!(json["code"], "ALERT_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_delete_alert_db_error() {
        let state = build_state::<FailingDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_alert_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/alert-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 500);
        let json = body_to_json(response).await;
        assert_eq!(json["code"], "DATABASE_ERROR");
    }
}
