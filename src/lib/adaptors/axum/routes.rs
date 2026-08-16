use axum::Router;
use axum::http::Response;
use axum::routing::{delete, get, patch, post, put};
use std::time::Duration;
use tower_http::trace::{DefaultMakeSpan, OnResponse, TraceLayer};
use tracing::{Level, Span};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adaptors::axum::app_state::AppState;
use crate::adaptors::axum::handlers::alert::{
    create_alert, delete_alert, get_alerts, get_daily_alerts, patch_alert, update_alert,
};
use crate::adaptors::axum::handlers::health::{liveness, readiness, startup};
use crate::adaptors::axum::handlers::meta::{get_app_config, get_raw_app_config};
use crate::adaptors::axum::openapi::ApiDoc;
use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::MetaPort;
use crate::domain::database::port::DatabasePort;

pub fn create_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/health", create_health_routes())
        .nest("/meta", create_meta_routes())
        .nest("/alerts", create_alert_routes())
        .merge(SwaggerUi::new("/swagger-ui/").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
                .on_response(StatusOnResponse),
        )
}

/// Logs each completed request, using `WARN` for client/server error statuses
/// (so they surface at the configured `info` trace level) and `DEBUG` for
/// successful responses.
#[derive(Clone, Copy)]
struct StatusOnResponse;

impl<B> OnResponse<B> for StatusOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let status = response.status();
        span.record("http.status_code", status.as_u16());
        span.record("http.latency", format!("{:.1?}", latency));

        if status.is_server_error() || status.is_client_error() {
            tracing::event!(
                parent: span,
                Level::WARN,
                status = %status,
                "request completed with error status"
            );
        } else {
            tracing::event!(
                parent: span,
                Level::DEBUG,
                status = %status,
                "request completed"
            );
        }
    }
}

pub fn create_health_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/startup", get(startup))
        .route("/readiness", get(readiness))
        .route("/liveness", get(liveness))
}

pub fn create_meta_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/raw_conf", get(get_raw_app_config))
        .route("/conf", get(get_app_config))
}

pub fn create_alert_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/", get(get_alerts))
        .route("/", post(create_alert))
        .route("/{identifier}", put(update_alert))
        .route("/{identifier}", patch(patch_alert))
        .route("/{identifier}", delete(delete_alert))
        .route("/daily", get(get_daily_alerts))
}

#[cfg(test)]
mod tests {
    use crate::test_support::{
        DEFAULT_PAGE_LIMIT, MockDb, MockMeta, PAGE_LIMIT_MAX, build_full_app, build_state,
        build_webserver,
    };
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get_status(uri: &str) -> u16 {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_full_app::<MockDb>(state);
        let response = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        response.status().as_u16()
    }

    #[tokio::test]
    async fn root_returns_hello_world() {
        assert_eq!(get_status("/").await, 200);
    }

    #[tokio::test]
    async fn health_routes_are_mounted() {
        assert_eq!(get_status("/health/startup").await, 200);
        assert_eq!(get_status("/health/readiness").await, 200);
        assert_eq!(get_status("/health/liveness").await, 200);
    }

    #[tokio::test]
    async fn meta_routes_are_mounted() {
        assert_eq!(get_status("/meta/conf").await, 200);
    }

    #[tokio::test]
    async fn alert_routes_are_mounted() {
        assert_eq!(get_status("/alerts").await, 200);
        assert_eq!(get_status("/alerts/daily").await, 200);
    }

    #[tokio::test]
    async fn test_routes_are_mounted() {
        assert_eq!(get_status("/meta/conf").await, 200);
    }

    #[test]
    fn malformed_request_body_logs_warn() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tracing::subscriber::with_default;

        struct CountingSubscriber(Arc<AtomicUsize>);

        impl tracing::Subscriber for CountingSubscriber {
            fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
                true
            }
            fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::Id {
                tracing::Id::from_u64(1)
            }
            fn record(&self, _span: &tracing::Id, _values: &tracing::span::Record<'_>) {}
            fn record_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id) {}
            fn event(&self, event: &tracing::Event<'_>) {
                if event.metadata().level() == &tracing::Level::WARN {
                    self.0.fetch_add(1, Ordering::SeqCst);
                }
            }
            fn enter(&self, _span: &tracing::Id) {}
            fn exit(&self, _span: &tracing::Id) {}
        }

        let warn_count = Arc::new(AtomicUsize::new(0));
        let subscriber = CountingSubscriber(warn_count.clone());

        with_default(subscriber, || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let state = build_state::<MockDb>(MockMeta::new(build_webserver(
                    DEFAULT_PAGE_LIMIT,
                    PAGE_LIMIT_MAX,
                )));
                let app = build_full_app::<MockDb>(state);
                let response = app
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/alerts")
                            .header("content-type", "application/json")
                            .body(Body::from("{not valid json"))
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(response.status().as_u16(), 422);
            });
        });

        assert_eq!(warn_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn openapi_json_is_served() {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_full_app::<MockDb>(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api-docs/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(doc["openapi"], "3.1.0");
        assert!(doc["paths"].get("/alerts").is_some());
        assert!(doc["paths"].get("/alerts/daily").is_some());
        assert!(
            doc["paths"]["/alerts/{identifier}"]["delete"]
                .get("operationId")
                .is_some()
        );
        assert!(doc["paths"].get("/meta/conf").is_some());
        assert!(doc["paths"].get("/meta/raw_conf").is_some());
        assert!(
            doc["components"]["schemas"]
                .get("AlertsListResponse")
                .is_some()
        );
        assert!(doc["components"]["schemas"].get("AlertSchema").is_some());
        assert!(
            doc["components"]["schemas"]
                .get("ApiErrorResponse")
                .is_some()
        );
        assert!(doc["components"]["schemas"].get("ErrorCode").is_some());
    }

    #[tokio::test]
    async fn swagger_ui_is_served() {
        assert_eq!(get_status("/swagger-ui/index.html").await, 200);
    }
}
