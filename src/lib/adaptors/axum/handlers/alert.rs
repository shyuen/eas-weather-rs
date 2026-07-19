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

#[cfg(test)]
mod tests {
    use std::future::Future;

    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;
    use axum::routing::get;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::adaptors::axum::app_state::AppState;
    use crate::domain::alert::port::*;
    use crate::domain::alert::service::AlertService;
    use crate::domain::config::model::*;
    use crate::domain::config::port::ConfigPort;
    use crate::domain::config::service::ConfigService;
    use crate::domain::database::model::Database;
    use crate::domain::database::port::DatabasePort;
    use crate::domain::database::service::DatabaseService;
    use crate::domain::logging::model::Logging;
    use crate::domain::meta::port::{MetaPort, ValidatedConfig};
    use crate::domain::webserver::model::Webserver;

    use super::{get_alerts, get_daily_alerts};

    const DEFAULT_PAGE_LIMIT: u64 = 10;
    const PAGE_LIMIT_MAX: u64 = 50;

    #[derive(Clone)]
    struct MockMeta {
        conf: ValidatedConfig,
    }

    impl MockMeta {
        fn new(webserver: Webserver) -> Self {
            let raw_log = ConfigLogging {
                format: None,
                trace_level: None,
            };
            let logging = Logging::new(&raw_log);
            let raw_db = ConfigDatabase {
                conn_url_file: None, conn_max_retries: None,
                conn_retry_init_delay_secs: None, conn_acquire_timeout_secs: None,
                conn_idle_timeout_secs: None, conn_max_lifetime_secs: None,
                max_connections: None, min_connections: None,
            };
            let database = Database::new(&raw_db);
            let conf = ValidatedConfig::new(logging, database, webserver);
            Self { conf }
        }
    }

    impl MetaPort for MockMeta {
        fn get_raw_config_data(&self) -> Config { unimplemented!() }
        fn get_conf(&self) -> ValidatedConfig { self.conf.clone() }
    }

    #[derive(Clone)]
    struct MockConfig {
        logging: Logging,
        database: Database,
        webserver: Webserver,
    }

    impl MockConfig {
        fn new() -> Self {
            let raw_logging = ConfigLogging { format: None, trace_level: None };
            let raw_database = ConfigDatabase {
                conn_url_file: None, conn_max_retries: None,
                conn_retry_init_delay_secs: None, conn_acquire_timeout_secs: None,
                conn_idle_timeout_secs: None, conn_max_lifetime_secs: None,
                max_connections: None, min_connections: None,
            };
            let raw_webserver = ConfigWebserver {
                hostname: None, port: None, base_path: None,
                shutdown_timeout_secs: None, api_key_file: None,
                jwt_key_file: None, jwt_access_token_expiry_secs: None,
                default_page_limit: None, page_limit_max: None,
            };
            Self {
                logging: Logging::new(&raw_logging),
                database: Database::new(&raw_database),
                webserver: Webserver::new(&raw_webserver),
            }
        }
    }

    impl ConfigPort for MockConfig {
        fn new() -> Self { Self::new() }
        fn get_raw_config(&self) -> &Config { unimplemented!() }
        fn get_logging_config(&self) -> &Logging { &self.logging }
        fn get_database_config(&self) -> &Database { &self.database }
        fn get_webserver_config(&self) -> &Webserver { &self.webserver }
        fn log_raw_config_input(&self) {}
        fn log_raw_config_validation(&self) {}
    }

    #[derive(Clone)]
    struct MockDb;

    impl DatabasePort for MockDb {
        fn new(_conf: &Database) -> Self { MockDb }
        fn log_adaptor_config(&self, _conf: &Database) {}
        fn create_pool(&mut self, _conf: &Database) -> impl Future<Output = ()> + Send { async {} }
        fn close_pool(&self) -> impl Future<Output = ()> + Send { async {} }
    }

    impl AlertPort for MockDb {
        fn get_latest_alerts_data(&self, _l: u64, _o: u64)
            -> impl Future<Output = Result<GetLatestAlertsResponse, GetLatestAlertsError>> + Send
        {
            async move { Ok(GetLatestAlertsResponse { alerts: vec![], total: 42 }) }
        }
        fn get_daily_alerts_data(&self, _l: u64, _o: u64)
            -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send
        {
            async move { Ok(GetDailyAlertsResponse { alerts: vec![], total: 42 }) }
        }
    }

    #[derive(Clone)]
    struct FailingDb;

    impl DatabasePort for FailingDb {
        fn new(_conf: &Database) -> Self { FailingDb }
        fn log_adaptor_config(&self, _conf: &Database) {}
        fn create_pool(&mut self, _conf: &Database) -> impl Future<Output = ()> + Send { async {} }
        fn close_pool(&self) -> impl Future<Output = ()> + Send { async {} }
    }

    impl AlertPort for FailingDb {
        fn get_latest_alerts_data(&self, _l: u64, _o: u64)
            -> impl Future<Output = Result<GetLatestAlertsResponse, GetLatestAlertsError>> + Send
        {
            async move { Err(GetLatestAlertsError::DatabaseError("test error".into())) }
        }
        fn get_daily_alerts_data(&self, _l: u64, _o: u64)
            -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send
        {
            async move { Err(GetDailyAlertsError::DatabaseError("test error".into())) }
        }
    }

    fn build_webserver(dpl: u64, plm: u64) -> Webserver {
        let raw = ConfigWebserver {
            hostname: None, port: None, base_path: None,
            shutdown_timeout_secs: None, api_key_file: None,
            jwt_key_file: None, jwt_access_token_expiry_secs: None,
            default_page_limit: Some(dpl), page_limit_max: Some(plm),
        };
        Webserver::new(&raw)
    }

    fn build_state<D>(webserver: Webserver) -> AppState<MockMeta, D>
    where
        D: DatabasePort + AlertPort + Clone,
    {
        let meta = MockMeta::new(webserver);
        let conf_service = ConfigService { port: MockConfig::new() };
        let db_service = DatabaseService::new(&conf_service);
        let alert_service = AlertService::new(db_service);
        AppState::new(meta, alert_service)
    }

    fn build_app<D>(state: AppState<MockMeta, D>) -> Router
    where
        D: DatabasePort + AlertPort + Clone + Send + Sync + 'static,
    {
        Router::new()
            .route("/", get(get_alerts::<MockMeta, D>))
            .route("/daily", get(get_daily_alerts::<MockMeta, D>))
            .with_state(state)
    }

    async fn body_to_json(response: axum::response::Response) -> serde_json::Value {
        let (_, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    // ── GET /alerts ──

    #[tokio::test]
    async fn test_get_alerts_default_params() {
        let state = build_state::<MockDb>(build_webserver(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX));
        let app = build_app(state);
        let response = app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
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
        let state = build_state::<MockDb>(build_webserver(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX));
        let app = build_app(state);
        let response = app.oneshot(Request::builder().uri("/?limit=100").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], PAGE_LIMIT_MAX);
    }

    #[tokio::test]
    async fn test_get_alerts_within_limit() {
        let state = build_state::<MockDb>(build_webserver(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX));
        let app = build_app(state);
        let response = app.oneshot(Request::builder().uri("/?limit=5").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], 5);
    }

    #[tokio::test]
    async fn test_get_alerts_with_offset() {
        let state = build_state::<MockDb>(build_webserver(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX));
        let app = build_app(state);
        let response = app.oneshot(Request::builder().uri("/?limit=5&offset=10").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], 5);
        assert_eq!(json["offset"], 10);
    }

    #[tokio::test]
    async fn test_get_alerts_error() {
        let state = build_state::<FailingDb>(build_webserver(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX));
        let app = build_app(state);
        let response = app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 500);
        let json = body_to_json(response).await;
        assert!(json["error"].is_string());
    }

    // ── GET /alerts/daily ──

    #[tokio::test]
    async fn test_get_daily_alerts_default_params() {
        let state = build_state::<MockDb>(build_webserver(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX));
        let app = build_app(state);
        let response = app.oneshot(Request::builder().uri("/daily").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], DEFAULT_PAGE_LIMIT);
        assert_eq!(json["offset"], 0);
        assert_eq!(json["total"], 42);
    }

    #[tokio::test]
    async fn test_get_daily_alerts_caps_limit() {
        let state = build_state::<MockDb>(build_webserver(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX));
        let app = build_app(state);
        let response = app.oneshot(Request::builder().uri("/daily?limit=100").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["limit"], PAGE_LIMIT_MAX);
    }

    #[tokio::test]
    async fn test_get_daily_alerts_error() {
        let state = build_state::<FailingDb>(build_webserver(DEFAULT_PAGE_LIMIT, PAGE_LIMIT_MAX));
        let app = build_app(state);
        let response = app.oneshot(Request::builder().uri("/daily").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 500);
        let json = body_to_json(response).await;
        assert!(json["error"].is_string());
    }
}
