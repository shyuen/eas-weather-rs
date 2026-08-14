use crate::adaptors::axum::app_state::AppState;
use crate::domain::alert::port::AlertPort;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;

/// Handler for GET /meta/raw_conf
pub(crate) async fn get_raw_app_config<MR, DR>(
    State(state): State<AppState<MR, DR>>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    let raw_conf = state.get_meta_port().get_raw_config_data();

    Json(json!({
        "raw_conf": raw_conf,
    }))
}

/// Handler for GET /meta/conf
pub(crate) async fn get_app_config<MR, DR>(
    State(state): State<AppState<MR, DR>>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    let conf = state.get_meta_port().get_conf();

    Json(json!({
        "conf": conf,
    }))
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
    use crate::domain::database::port::DatabasePort;
    use crate::test_support::{MockDb, MockMeta, build_state, build_webserver};

    use super::{get_app_config, get_raw_app_config};

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

    fn build_meta_app<D>(state: AppState<MockMeta, D>) -> axum::Router
    where
        D: DatabasePort + AlertPort + Clone + Send + Sync + 'static,
    {
        axum::Router::new()
            .route("/raw_conf", get(get_raw_app_config::<MockMeta, D>))
            .route("/conf", get(get_app_config::<MockMeta, D>))
            .with_state(state)
    }

    async fn body_to_json(response: axum::response::Response) -> serde_json::Value {
        let (_, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_get_raw_app_config() {
        let meta = MockMeta::new(build_webserver(10, 100)).with_raw_config(raw_conf());
        let state = build_state::<MockDb>(meta);
        let app = build_meta_app::<MockDb>(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/raw_conf")
                    .body(Body::empty())
                    .unwrap(),
            )
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
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(10, 100)));
        let app = build_meta_app::<MockDb>(state);
        let response = app
            .oneshot(Request::builder().uri("/conf").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let json = body_to_json(response).await;
        assert!(json["conf"]["conf_webserver"].is_object());
        assert_eq!(json["conf"]["conf_webserver"]["default_page_limit"], 10);
        assert_eq!(json["conf"]["conf_webserver"]["page_limit_max"], 100);
    }
}
