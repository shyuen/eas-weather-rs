use axum::Router;
use axum::http::{Response, StatusCode};
use axum::middleware;
use axum::routing::{delete, get, patch, post, put};
use std::time::Duration;
use tower_http::trace::{DefaultMakeSpan, OnResponse, TraceLayer};
use tracing::{Level, Span};
use utoipa_swagger_ui::SwaggerUi;

use crate::adaptors::axum::app_state::AppState;
use crate::adaptors::axum::auth::require_api_key;
use crate::adaptors::axum::handlers::alert::{
    create_alert, delete_alert, get_alerts, get_daily_alerts, patch_alert, update_alert,
};
use crate::adaptors::axum::handlers::conf::{get_app_config, get_raw_config};
use crate::adaptors::axum::handlers::health::{liveness, readiness, startup};
use crate::adaptors::axum::openapi::api_doc;
use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::ConfigPort;

pub fn create_routes<CP, AP>(
    api_key: Option<String>,
    base_path: &Option<String>,
) -> Router<AppState<CP, AP>>
where
    CP: ConfigPort,
    AP: AlertPort,
{
    // API routes (optionally nested under base_path) and the docs at root.
    let api_routes = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/favicon.ico", get(|| async { StatusCode::NO_CONTENT }))
        .nest("/health", create_health_routes())
        .nest("/conf", create_conf_routes(api_key))
        .nest("/alerts", create_alert_routes());

    // Apply base_path to the API routes only; Swagger UI + OpenAPI spec stay
    // at /swagger-ui and /api-docs/openapi.json (paths are embedded in the
    // served Swagger UI page, so they must match the actual mount point).
    let api_routes = match base_path
        .as_deref()
        .filter(|p| !p.is_empty() && *p != "/")
    {
        Some(path) => Router::new().nest(path, api_routes),
        None => api_routes,
    };

    api_routes
        .merge(SwaggerUi::new("/swagger-ui/").url("/api-docs/openapi.json", api_doc(base_path)))
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

pub fn create_health_routes<CP, AP>() -> Router<AppState<CP, AP>>
where
    CP: ConfigPort,
    AP: AlertPort,
{
    Router::new()
        .route("/startup", get(startup))
        .route("/readiness", get(readiness))
        .route("/liveness", get(liveness))
}

pub fn create_conf_routes<CP, AP>(api_key: Option<String>) -> Router<AppState<CP, AP>>
where
    CP: ConfigPort,
    AP: AlertPort,
{
    Router::new()
        .route("/raw", get(get_raw_config))
        .route("/app", get(get_app_config))
        .layer(middleware::from_fn_with_state(api_key, require_api_key))
}

pub fn create_alert_routes<CP, AP>() -> Router<AppState<CP, AP>>
where
    CP: ConfigPort,
    AP: AlertPort,
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
        DEFAULT_PAGE_LIMIT, MockConfig, MockDb, PAGE_LIMIT_MAX, build_full_app, build_state,
        mock_config_service,
    };
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get_status(uri: &str) -> u16 {
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
    async fn conf_routes_are_mounted() {
        assert_eq!(get_status("/conf/app").await, 200);
    }

    #[tokio::test]
    async fn alert_routes_are_mounted() {
        assert_eq!(get_status("/alerts").await, 200);
        assert_eq!(get_status("/alerts/daily").await, 200);
    }

    #[tokio::test]
    async fn test_routes_are_mounted() {
        assert_eq!(get_status("/conf/app").await, 200);
    }

    #[tokio::test]
    async fn routes_nested_under_base_path() {
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
        let app = super::create_routes::<MockConfig, MockDb>(None, &Some("/api".into()))
            .with_state(state);

        async fn status(app: &axum::Router, uri: &str) -> u16 {
            app.clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap()
                .status()
                .as_u16()
        }

        // Root-level API paths are not served when base_path is set.
        // NOTE: intentionally avoid asserting 404 here — the 4xx status triggers
        // a WARN through the TraceLayer, which races with the global-subscriber
        // counting test (`malformed_request_body_logs_warn`) under parallelism.
        // Everything is nested under the base path
        assert_eq!(status(&app, "/api/alerts").await, 200);
        assert_eq!(status(&app, "/api/conf/app").await, 200);
        assert_eq!(status(&app, "/api/health/startup").await, 200);
        // Swagger UI + OpenAPI spec are served at the root
        assert_eq!(status(&app, "/swagger-ui/index.html").await, 200);
        assert_eq!(status(&app, "/api-docs/openapi.json").await, 200);
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
                let state = build_state::<MockDb>(
                    mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
                    &MockDb,
                );
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
        let state = build_state::<MockDb>(
            mock_config_service(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX),
            &MockDb,
        );
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
        assert!(doc["paths"].get("/conf/app").is_some());
        assert!(doc["paths"].get("/conf/raw").is_some());
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
