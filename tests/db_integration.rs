//! Integration tests for the full HTTP stack against a real MySQL/MariaDB
//! database, using the real `DatabaseMySql` adaptor (no mocks).
//!
//! These tests are `#[ignore]`-gated so plain `cargo test` (and the CI `check`
//! job) stays offline. Run them with a reachable database:
//!
//! ```text
//! EAS_WEATHER_RS_TEST_DB=mysql://root:root@localhost:3306/eas_weather \
//!   cargo test --test db_integration -- --ignored
//! ```
//!
//! The connection URL resolves in this order:
//!   1. `EAS_WEATHER_RS_TEST_DB` env var (raw URL)
//!   2. the `config/mysql_conn_url` secret file (gitignored dev default)
//!   3. the packaged default (`DbConnectionString::default()`)

use std::io::Write as _;

use axum::Router;
use axum::http::{Request, StatusCode, header};

use eas_weather_rs::adaptors::axum::app_state::AppState;
use eas_weather_rs::adaptors::axum::routes::create_routes;
use eas_weather_rs::adaptors::xsqlx::model::DatabaseMySql;
use eas_weather_rs::domain::alert::service::AlertService;
use eas_weather_rs::domain::config::model::{
    Config, ConfigDatabase, ConfigLogging, ConfigWebserver, RawConfigInputs,
};
use eas_weather_rs::domain::config::port::ConfigPort;
use eas_weather_rs::domain::config::service::ConfigService;
use eas_weather_rs::domain::database::model::Database;
use eas_weather_rs::domain::database::new_types::db_conn_string::DbConnectionString;
use eas_weather_rs::domain::database::service::DatabaseService;
use eas_weather_rs::domain::logging::model::Logging;
use eas_weather_rs::domain::logging::new_types::lg_format::LoggingFormat;
use eas_weather_rs::domain::logging::new_types::lg_trace_level::LoggingTraceLevel;
use eas_weather_rs::domain::webserver::model::Webserver;
use http_body_util::BodyExt;
use sqlx::Connection;
use tower::ServiceExt;

type TestState = AppState<TestConfigPort, DatabaseMySql, DatabaseMySql>;

/// `ConfigPort` double for the integration tests. It wraps the *real* domain
/// models (`Logging`, `Database`, `Webserver`) built from a controlled raw
/// config, so handlers resolve the same validated config they would in
/// production (including the webserver page limits used by the list handlers).
#[derive(Clone)]
struct TestConfigPort {
    raw: Config,
    logging: Logging,
    database: Database,
    webserver: Webserver,
}

impl TestConfigPort {
    fn new(conn_url: &str) -> Self {
        let conn_url_file = write_conn_url_file(conn_url);

        let raw = Config {
            logging: ConfigLogging {
                format: Some("text".into()),
                trace_level: Some("info".into()),
            },
            webserver: ConfigWebserver {
                hostname: None,
                port: None,
                base_path: None,
                shutdown_timeout_secs: None,
                api_key_file: None,
                jwt_key_file: None,
                jwt_access_token_expiry_secs: None,
                default_page_limit: Some(25),
                page_limit_max: Some(100),
            },
            database: ConfigDatabase {
                conn_url_file: Some(conn_url_file.to_string_lossy().into_owned()),
                conn_max_retries: Some(3),
                conn_retry_init_delay_secs: Some(1),
                conn_acquire_timeout_secs: Some(30),
                conn_idle_timeout_secs: Some(300),
                conn_max_lifetime_secs: Some(1800),
                max_connections: Some(10),
                min_connections: Some(1),
            },
            config_file: None,
        };

        Self {
            raw: raw.clone(),
            logging: Logging::new(LoggingFormat::default(), LoggingTraceLevel::default()),
            database: Database::new(&raw.database),
            webserver: Webserver::new(&raw.webserver),
        }
    }
}

impl ConfigPort for TestConfigPort {
    fn new() -> Self {
        // The integration tests always construct with an explicit conn URL via
        // `TestConfigPort::new`; the trait's `new()` only needs to be *some*
        // valid value. It points at a placeholder file whose contents are never
        // used (no pool is built through this path).
        let conn_url_file = write_conn_url_file("mysql://root@localhost:3306/placeholder");
        let raw = Config {
            logging: ConfigLogging {
                format: Some("text".into()),
                trace_level: Some("info".into()),
            },
            webserver: ConfigWebserver {
                hostname: None,
                port: None,
                base_path: None,
                shutdown_timeout_secs: None,
                api_key_file: None,
                jwt_key_file: None,
                jwt_access_token_expiry_secs: None,
                default_page_limit: Some(25),
                page_limit_max: Some(100),
            },
            database: ConfigDatabase {
                conn_url_file: Some(conn_url_file.to_string_lossy().into_owned()),
                conn_max_retries: Some(3),
                conn_retry_init_delay_secs: Some(1),
                conn_acquire_timeout_secs: Some(30),
                conn_idle_timeout_secs: Some(300),
                conn_max_lifetime_secs: Some(1800),
                max_connections: Some(10),
                min_connections: Some(1),
            },
            config_file: None,
        };
        Self {
            raw: raw.clone(),
            logging: Logging::new(LoggingFormat::default(), LoggingTraceLevel::default()),
            database: Database::new(&raw.database),
            webserver: Webserver::new(&raw.webserver),
        }
    }

    fn get_raw_config(&self) -> &Config {
        &self.raw
    }

    fn get_logging_config(&self) -> &Logging {
        &self.logging
    }

    fn get_database_config(&self) -> &Database {
        &self.database
    }

    fn get_webserver_config(&self) -> &Webserver {
        &self.webserver
    }

    fn raw_config_input(&self) -> RawConfigInputs {
        RawConfigInputs {
            cli: serde_json::Value::Null,
            env: serde_json::Value::Null,
            files: serde_json::Value::Null,
            final_config: serde_json::Value::Null,
        }
    }

    fn validate_raw_config(&self) -> Vec<eas_weather_rs::domain::config::issue::ConfigIssue> {
        Vec::new()
    }
}

/// Resolve the database connection URL for the integration tests.
fn resolve_conn_url() -> String {
    // 1. Explicit override from the caller.
    if let Ok(url) = std::env::var("EAS_WEATHER_RS_TEST_DB") {
        let url = url.trim().to_string();
        if !url.is_empty() {
            return url;
        }
    }

    // 2. The gitignored dev secret file, if present.
    if let Ok(url) = std::fs::read_to_string("config/mysql_conn_url") {
        let url = url.trim().to_string();
        if !url.is_empty() {
            return url;
        }
    }

    // 3. Packaged default.
    DbConnectionString::default().get().to_string()
}

/// Write the resolved connection URL to a temp file so the real
/// `DbConnectionString::new` secret-file plumbing is exercised.
fn write_conn_url_file(conn_url: &str) -> std::path::PathBuf {
    let mut file = tempfile::NamedTempFile::new()
        .expect("failed to create temp conn url file for integration tests");
    file.write_all(conn_url.as_bytes())
        .expect("failed to write conn url");
    file.into_temp_path()
        .keep()
        .expect("failed to persist conn url file")
}

/// Ensure the migrations have been applied before the test exercises the API.
///
/// The migration SQL is idempotent (`CREATE TABLE IF NOT EXISTS`), and sqlx
/// takes a MySQL advisory lock around `run()`, so calling this from each test
/// is safe even under parallel execution.
async fn ensure_migrated(conn_url: &str) {
    let mut conn = sqlx::MySqlConnection::connect(conn_url)
        .await
        .expect("failed to connect for migrations: check EAS_WEATHER_RS_TEST_DB");
    sqlx::migrate!("./migrations")
        .run(&mut conn)
        .await
        .expect("failed to apply migrations");
}

/// Build the full application (config, alert service, real DB adaptor, HTTP
/// router) wired like the composition root, with the pool connected.
async fn build_app(conn_url: &str) -> Router {
    ensure_migrated(conn_url).await;

    let config = TestConfigPort::new(conn_url);
    let database_conf = config.database.clone();

    // Create and connect the real pool through the DatabaseService wrapper.
    let mut db_service = DatabaseService::<DatabaseMySql>::new(database_conf);
    db_service
        .create_pool()
        .await
        .expect("failed to create database pool");

    let db = db_service.get_database_port().clone();
    let alert_service = AlertService::new(&db);
    let config_service = ConfigService::<TestConfigPort>::from_config_port(config);
    let state = TestState::new(config_service, alert_service, db);

    create_routes::<TestConfigPort, DatabaseMySql, DatabaseMySql>(None, &None).with_state(state)
}

/// Send a JSON request and collect the response status + parsed body.
async fn send_json(
    app: &Router,
    method: axum::http::Method,
    uri: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            body.map(|v| v.to_string()).unwrap_or_default(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    };
    (status, json)
}

/// A unique identifier for the current test's created alert, so parallel test
/// runs never collide on the primary key.
fn unique_identifier() -> String {
    format!(
        "itest-{}",
        time::OffsetDateTime::now_utc().unix_timestamp_nanos()
    )
}

/// A valid create body with the given identifier and sent timestamp.
fn create_body(identifier: &str, sent: &str) -> serde_json::Value {
    serde_json::json!({
        "identifier": identifier,
        "sender": "Sender123",
        "sent": sent,
        "status": "Actual",
        "msg_type": "Alert",
        "source": "Weather Station 1",
        "scope": "Public",
        "references": ["Sender1,Alert123,2024-06-01T12:00:00-00:00"]
    })
}

fn now_rfc3339() -> String {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("failed to format current time")
}

#[tokio::test]
#[ignore = "requires a running MySQL/MariaDB; set EAS_WEATHER_RS_TEST_DB"]
async fn health_endpoints_pass_with_working_database() {
    let conn_url = resolve_conn_url();
    let app = build_app(&conn_url).await;

    let (status, _) = send_json(&app, axum::http::Method::GET, "/health/startup", None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_json(&app, axum::http::Method::GET, "/health/liveness", None).await;
    assert_eq!(status, StatusCode::OK);
    // Readiness exercises the real pool with `SELECT 1`.
    let (status, _) = send_json(&app, axum::http::Method::GET, "/health/readiness", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires a running MySQL/MariaDB; set EAS_WEATHER_RS_TEST_DB"]
async fn alert_lifecycle_persists_and_round_trips() {
    let conn_url = resolve_conn_url();
    let app = build_app(&conn_url).await;

    let identifier = unique_identifier();

    // POST /alerts → 201, persisted with the same identifier.
    let (status, body) = send_json(
        &app,
        axum::http::Method::POST,
        "/alerts",
        Some(create_body(&identifier, "2002-05-24T16:49:00-00:00")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["identifier"], identifier);

    // GET /alerts list reflects the persisted row (real SQL, ROW_NUMBER filter).
    let (status, list) = send_json(&app, axum::http::Method::GET, "/alerts?limit=100", None).await;
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<&str> = list["alerts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["identifier"].as_str().unwrap())
        .collect();
    assert!(
        ids.contains(&identifier.as_str()),
        "created alert missing from list"
    );

    // PATCH /alerts/{id} → 200, status updated.
    let (status, patched) = send_json(
        &app,
        axum::http::Method::PATCH,
        &format!("/alerts/{identifier}"),
        Some(serde_json::json!({ "status": "Test" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patched["status"], "Test");

    // PUT /alerts/{id} → 200, full replacement.
    let (status, replaced) = send_json(
        &app,
        axum::http::Method::PUT,
        &format!("/alerts/{identifier}"),
        Some(serde_json::json!({
            "sender": "Sender456",
            "sent": "2003-06-30T08:15:00+00:00",
            "status": "Exercise",
            "msg_type": "Update",
            "source": "Station Two",
            "scope": "Restricted",
            "references": []
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replaced["sender"], "Sender456");
    assert_eq!(replaced["msg_type"], "Update");

    // DELETE /alerts/{id} → 204, then a second delete → 404.
    let (status, _) = send_json(
        &app,
        axum::http::Method::DELETE,
        &format!("/alerts/{identifier}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = send_json(
        &app,
        axum::http::Method::DELETE,
        &format!("/alerts/{identifier}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // The deleted alert must no longer appear in the list.
    let (_, list) = send_json(&app, axum::http::Method::GET, "/alerts?limit=100", None).await;
    let ids: Vec<&str> = list["alerts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["identifier"].as_str().unwrap())
        .collect();
    assert!(
        !ids.contains(&identifier.as_str()),
        "deleted alert still in list"
    );
}

#[tokio::test]
#[ignore = "requires a running MySQL/MariaDB; set EAS_WEATHER_RS_TEST_DB"]
async fn invalid_alert_is_rejected_with_validation_error() {
    let conn_url = resolve_conn_url();
    let app = build_app(&conn_url).await;

    let identifier = unique_identifier();
    let mut body = create_body(&identifier, "2002-05-24T16:49:00-00:00");
    body["sender"] = serde_json::json!("Invalid Sender");

    let (status, err) = send_json(&app, axum::http::Method::POST, "/alerts", Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(err["code"], "ALERT_VALIDATION_FAILED");
}

#[tokio::test]
#[ignore = "requires a running MySQL/MariaDB; set EAS_WEATHER_RS_TEST_DB"]
async fn daily_endpoint_returns_only_today_alerts() {
    let conn_url = resolve_conn_url();
    let app = build_app(&conn_url).await;

    let today_id = unique_identifier();
    let old_id = unique_identifier();

    // An alert sent now is within today's window (CURDATE() .. +1 day).
    let (status, _) = send_json(
        &app,
        axum::http::Method::POST,
        "/alerts",
        Some(create_body(&today_id, &now_rfc3339())),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // An alert sent long ago must be excluded by the daily filter.
    let (status, _) = send_json(
        &app,
        axum::http::Method::POST,
        "/alerts",
        Some(create_body(&old_id, "2002-05-24T16:49:00-00:00")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, daily) = send_json(
        &app,
        axum::http::Method::GET,
        "/alerts/daily?limit=100",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let ids: Vec<&str> = daily["alerts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["identifier"].as_str().unwrap())
        .collect();
    assert!(
        ids.contains(&today_id.as_str()),
        "today alert missing from daily"
    );
    assert!(
        !ids.contains(&old_id.as_str()),
        "old alert should be excluded by daily filter"
    );

    // Clean up the rows created by this test.
    for id in [&today_id, &old_id] {
        send_json(
            &app,
            axum::http::Method::DELETE,
            &format!("/alerts/{id}"),
            None,
        )
        .await;
    }
}
