use crate::adaptors::axum::app_state::AppState;
use crate::domain::alert::port::AlertPort;
use crate::domain::config::model::Config;
use crate::domain::config::model::ValidatedConfig;
use crate::domain::config::port::ConfigPort;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Serialize;
use utoipa::ToSchema;

/// Raw configuration dump returned by `/conf/raw`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RawConfResponse {
    #[schema(value_type = Object)]
    pub raw_conf: Config,
}

/// Processed configuration returned by `/conf/app`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConfResponse {
    #[schema(value_type = Object)]
    pub conf: ValidatedConfig,
}

/// Handler for GET /conf/raw
///
/// Returns the raw configuration combined data source from CLI, ENV, and config files as a JSON object.
/// This endpoint is useful for debugging and inspecting the application's configuration.
#[utoipa::path(
    get,
    path = "/conf/raw",
    responses(
        (status = 200, description = "Raw configuration", body = RawConfResponse)
    ),
    tag = "conf"
)]
pub(crate) async fn get_raw_config<CP, AP>(
    State(state): State<AppState<CP, AP>>,
) -> impl IntoResponse
where
    CP: ConfigPort,
    AP: AlertPort,
{
    let raw_conf = state.get_config_service().get_raw_config().clone();

    Json(RawConfResponse { raw_conf })
}

/// Handler for GET /conf/app
///
/// Returns a validated configuration struct that is used by the application, which may differ from the raw configuration due to validation and default values.
#[utoipa::path(
    get,
    path = "/conf/app",
    responses(
        (status = 200, description = "Processed configuration", body = ConfResponse)
    ),
    tag = "conf"
)]
pub(crate) async fn get_app_config<CP, AP>(
    State(state): State<AppState<CP, AP>>,
) -> impl IntoResponse
where
    CP: ConfigPort,
    AP: AlertPort,
{
    let conf = state.get_config_service().get_validated_app_conf();

    Json(ConfResponse { conf })
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::adaptors::axum::app_state::AppState;
    use crate::domain::alert::port::AlertPort;
    use crate::domain::config::model::*;
    use crate::domain::config::service::ConfigService;
    use crate::test_support::{
        MockConfig, MockDb, build_state, build_webserver, mock_config_service,
    };

    use super::{get_app_config, get_raw_config};

    fn raw_conf() -> Config {
        Config {
            logging: ConfigLogging {
                format: Some("json".into()),
                trace_level: Some("debug".into()),
            },
            webserver: ConfigWebserver {
                hostname: Some("0.0.0.0".into()),
                port: Some(8080),
                base_path: Some("/api".into()),
                shutdown_timeout_secs: Some(30),
                api_key_file: Some("key.pem".into()),
                jwt_key_file: Some("jwt.pem".into()),
                jwt_access_token_expiry_secs: Some(3600),
                default_page_limit: Some(10),
                page_limit_max: Some(100),
            },
            database: ConfigDatabase {
                conn_url_file: Some("db.pem".into()),
                conn_max_retries: Some(5),
                conn_retry_init_delay_secs: Some(1),
                conn_acquire_timeout_secs: Some(2),
                conn_idle_timeout_secs: Some(3),
                conn_max_lifetime_secs: Some(4),
                max_connections: Some(6),
                min_connections: Some(1),
            },
            config_file: Some("config/default.toml".into()),
        }
    }

    fn build_conf_app<D>(state: AppState<MockConfig, D>) -> axum::Router
    where
        D: AlertPort + Clone + Send + Sync + 'static,
    {
        axum::Router::new()
            .route("/raw", get(get_raw_config::<MockConfig, D>))
            .route("/app", get(get_app_config::<MockConfig, D>))
            .with_state(state)
    }

    async fn body_to_json(response: axum::response::Response) -> serde_json::Value {
        let (_, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_get_raw_config() {
        let config_service = ConfigService::from_config_port(
            MockConfig::new()
                .with_webserver_config(build_webserver(10, 100))
                .with_raw_config(raw_conf()),
        );
        let state = build_state::<MockDb>(config_service, &MockDb);
        let app = build_conf_app::<MockDb>(state);
        let response = app
            .oneshot(Request::builder().uri("/raw").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert_eq!(json["raw_conf"]["logging"]["format"], "json");
        assert_eq!(json["raw_conf"]["logging"]["trace_level"], "debug");
        assert_eq!(json["raw_conf"]["webserver"]["hostname"], "0.0.0.0");
        assert_eq!(json["raw_conf"]["webserver"]["port"], 8080);
        assert_eq!(json["raw_conf"]["webserver"]["base_path"], "/api");
        assert_eq!(json["raw_conf"]["webserver"]["default_page_limit"], 10);
        assert_eq!(json["raw_conf"]["webserver"]["page_limit_max"], 100);
        assert_eq!(json["raw_conf"]["database"]["conn_max_retries"], 5);
        assert_eq!(json["raw_conf"]["database"]["max_connections"], 6);
        assert_eq!(json["raw_conf"]["config_file"], "config/default.toml");
    }

    #[tokio::test]
    async fn test_get_app_config() {
        let state = build_state::<MockDb>(mock_config_service(10, 100), &MockDb);
        let app = build_conf_app::<MockDb>(state);
        let response = app
            .oneshot(Request::builder().uri("/app").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert!(json["conf"]["conf_webserver"].is_object());
        assert_eq!(json["conf"]["conf_webserver"]["default_page_limit"], 10);
        assert_eq!(json["conf"]["conf_webserver"]["page_limit_max"], 100);
    }
}
