//! Shared test doubles and helpers for unit tests across the crate.
//!
//! Only compiled under `#[cfg(test)]`. Mocks implement the domain port traits
//! so generic services and axum handlers can be exercised without real adaptors.
//!
//! The mock port methods deliberately mirror the domain port traits' `impl
//! Future + Send` signatures (static dispatch — see AGENTS.md), so the
//! `manual_async_fn` lint is intentionally not applied here.
#![allow(clippy::manual_async_fn)]

use std::future::Future;

use crate::domain::alert::model::Alert;
use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::port::*;
use crate::domain::alert::service::AlertService;
use crate::domain::config::adaptor_config::{AdaptorConfigField, AdaptorConfigRepr};
use crate::domain::config::model::*;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::model::Database;
use crate::domain::database::port::{DatabaseCloseError, DatabaseConnectError, DatabasePort};
use crate::domain::database::service::DatabaseService;
use crate::domain::logging::model::Logging;
use crate::domain::meta::port::{MetaPort, ValidatedConfig};
use crate::domain::webserver::model::Webserver;

pub const DEFAULT_PAGE_LIMIT: u64 = 10;
pub const PAGE_LIMIT_MAX: u64 = 50;

fn raw_logging() -> ConfigLogging {
    ConfigLogging {
        format: None,
        trace_level: None,
    }
}

fn raw_database() -> ConfigDatabase {
    ConfigDatabase {
        conn_url_file: None,
        conn_max_retries: None,
        conn_retry_init_delay_secs: None,
        conn_acquire_timeout_secs: None,
        conn_idle_timeout_secs: None,
        conn_max_lifetime_secs: None,
        max_connections: None,
        min_connections: None,
    }
}

fn raw_webserver(dpl: Option<u64>, plm: Option<u64>) -> ConfigWebserver {
    ConfigWebserver {
        hostname: None,
        port: None,
        base_path: None,
        shutdown_timeout_secs: None,
        api_key_file: None,
        jwt_key_file: None,
        jwt_access_token_expiry_secs: None,
        default_page_limit: dpl,
        page_limit_max: plm,
    }
}

/// Builds a `Webserver` with the given default page limit and page limit max.
pub fn build_webserver(dpl: u64, plm: u64) -> Webserver {
    Webserver::new(&raw_webserver(Some(dpl), Some(plm)))
}

/// `MetaPort` double that always returns a `ValidatedConfig` built from the
/// supplied `Webserver` (with neutral logging/database config), and an
/// optional raw `Config`.
#[derive(Clone)]
pub struct MockMeta {
    conf: ValidatedConfig,
    raw_conf: Option<Config>,
}

impl MockMeta {
    pub fn new(webserver: Webserver) -> Self {
        let logging = Logging::new(&raw_logging());
        let database = Database::new(&raw_database());
        let conf = ValidatedConfig::new(logging, database, webserver);
        Self {
            conf,
            raw_conf: None,
        }
    }

    pub fn with_raw_config(mut self, raw: Config) -> Self {
        self.raw_conf = Some(raw);
        self
    }
}

impl MetaPort for MockMeta {
    fn get_raw_config_data(&self) -> Config {
        self.raw_conf
            .clone()
            .expect("MockMeta::with_raw_config must be called to use get_raw_config_data")
    }
    fn get_conf(&self) -> ValidatedConfig {
        self.conf.clone()
    }
}

/// `ConfigPort` double that always returns neutral (default) config.
#[derive(Clone)]
pub struct MockConfig {
    logging: Logging,
    database: Database,
    webserver: Webserver,
}

impl MockConfig {
    pub fn new() -> Self {
        Self {
            logging: Logging::new(&raw_logging()),
            database: Database::new(&raw_database()),
            webserver: Webserver::new(&raw_webserver(None, None)),
        }
    }
}

impl ConfigPort for MockConfig {
    fn new() -> Self {
        Self::new()
    }
    fn get_raw_config(&self) -> &Config {
        unimplemented!("not needed by tests")
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
    fn raw_config_input(&self) -> crate::domain::config::model::RawConfigInputs {
        crate::domain::config::model::RawConfigInputs {
            cli: serde_json::Value::Null,
            env: serde_json::Value::Null,
            files: serde_json::Value::Null,
            final_config: serde_json::Value::Null,
        }
    }
    fn validate_raw_config(&self) -> Vec<crate::domain::config::issue::ConfigIssue> {
        Vec::new()
    }
}

/// `DatabasePort + AlertPort` double that always returns successful results.
#[derive(Clone)]
pub struct MockDb;

/// Build a valid [`Alert`] with deterministic values, used by the success-path
/// mocks (`MockDb`, `FlakyDb`) when an existing alert must be returned.
fn mock_alert(identifier: AlertIdentifier) -> Alert {
    use crate::domain::alert::new_types::alert_msg_type::AlertMsgType;
    use crate::domain::alert::new_types::alert_references::AlertReferences;
    use crate::domain::alert::new_types::alert_references::ExtendedMessageIdentifier;
    use crate::domain::alert::new_types::alert_scope::AlertScope;
    use crate::domain::alert::new_types::alert_sender::AlertSender;
    use crate::domain::alert::new_types::alert_sent::AlertSent;
    use crate::domain::alert::new_types::alert_source::AlertSource;
    use crate::domain::alert::new_types::alert_status::AlertStatus;
    use time::OffsetDateTime;

    let sender = AlertSender::new("Sender123".to_string()).unwrap();
    let sent = AlertSent::new(OffsetDateTime::now_utc()).unwrap();
    let status = AlertStatus::new("Actual".to_string()).unwrap();
    let msg_type = AlertMsgType::new("Alert".to_string()).unwrap();
    let source = AlertSource::new("Weather Station 1").unwrap();
    let scope = AlertScope::new("Public".to_string()).unwrap();
    let reference =
        ExtendedMessageIdentifier::new("Sender1,Alert123,2024-06-01T12:00:00-00:00").unwrap();
    let references = AlertReferences::new(vec![reference]).unwrap();

    Alert::new(
        identifier, sender, sent, status, msg_type, source, scope, references,
    )
}

impl DatabasePort for MockDb {
    fn new(_conf: &Database) -> Self {
        MockDb
    }
    fn create_pool(&mut self) -> impl Future<Output = Result<(), DatabaseConnectError>> + Send {
        async { Ok(()) }
    }
    fn close_pool(&self) -> impl Future<Output = Result<(), DatabaseCloseError>> + Send {
        async { Ok(()) }
    }
}

impl AdaptorConfigRepr for MockDb {
    fn adaptor_name(&self) -> &'static str {
        "mock_db"
    }
    fn config_fields(&self) -> Vec<AdaptorConfigField> {
        vec![]
    }
}

impl AlertPort for MockDb {
    fn get_latest_alerts_data(
        &self,
        _l: u64,
        _o: u64,
    ) -> impl Future<Output = Result<GetLatestAlertsResponse, GetLatestAlertsError>> + Send {
        async move {
            Ok(GetLatestAlertsResponse {
                alerts: vec![],
                total: 42,
            })
        }
    }
    fn get_daily_alerts_data(
        &self,
        _l: u64,
        _o: u64,
    ) -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send {
        async move {
            Ok(GetDailyAlertsResponse {
                alerts: vec![],
                total: 42,
            })
        }
    }
    fn create_alert_data(
        &self,
        alert: Alert,
    ) -> impl Future<Output = Result<CreateAlertResponse, CreateAlertError>> + Send {
        async move { Ok(CreateAlertResponse { alert }) }
    }
    fn update_alert_data(
        &self,
        _identifier: &AlertIdentifier,
        alert: Alert,
    ) -> impl Future<Output = Result<UpdateAlertResponse, UpdateAlertError>> + Send {
        async move { Ok(UpdateAlertResponse { alert }) }
    }
    fn get_alert_data(
        &self,
        identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<GetAlertResponse, GetAlertError>> + Send {
        let identifier = identifier.clone();
        async move {
            Ok(GetAlertResponse {
                alert: mock_alert(identifier),
            })
        }
    }
    fn delete_alert_data(
        &self,
        _identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<DeleteAlertResponse, DeleteAlertError>> + Send {
        async move { Ok(DeleteAlertResponse) }
    }
}

/// `DatabasePort + AlertPort` double that always fails with a database error.
#[derive(Clone)]
pub struct FailingDb;

impl DatabasePort for FailingDb {
    fn new(_conf: &Database) -> Self {
        FailingDb
    }
    fn create_pool(&mut self) -> impl Future<Output = Result<(), DatabaseConnectError>> + Send {
        async { Ok(()) }
    }
    fn close_pool(&self) -> impl Future<Output = Result<(), DatabaseCloseError>> + Send {
        async { Ok(()) }
    }
}

impl AdaptorConfigRepr for FailingDb {
    fn adaptor_name(&self) -> &'static str {
        "failing_db"
    }
    fn config_fields(&self) -> Vec<AdaptorConfigField> {
        vec![]
    }
}

impl AlertPort for FailingDb {
    fn get_latest_alerts_data(
        &self,
        _l: u64,
        _o: u64,
    ) -> impl Future<Output = Result<GetLatestAlertsResponse, GetLatestAlertsError>> + Send {
        async move { Err(GetLatestAlertsError::DatabaseError("test error".into())) }
    }
    fn get_daily_alerts_data(
        &self,
        _l: u64,
        _o: u64,
    ) -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send {
        async move { Err(GetDailyAlertsError::DatabaseError("test error".into())) }
    }
    fn create_alert_data(
        &self,
        _alert: Alert,
    ) -> impl Future<Output = Result<CreateAlertResponse, CreateAlertError>> + Send {
        async move { Err(CreateAlertError::DatabaseError("test error".into())) }
    }
    fn update_alert_data(
        &self,
        _identifier: &AlertIdentifier,
        _alert: Alert,
    ) -> impl Future<Output = Result<UpdateAlertResponse, UpdateAlertError>> + Send {
        async move { Err(UpdateAlertError::DatabaseError("test error".into())) }
    }
    fn get_alert_data(
        &self,
        _identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<GetAlertResponse, GetAlertError>> + Send {
        async move { Err(GetAlertError::DatabaseError("test error".into())) }
    }
    fn delete_alert_data(
        &self,
        _identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<DeleteAlertResponse, DeleteAlertError>> + Send {
        async move { Err(DeleteAlertError::DatabaseError("test error".into())) }
    }
}

/// `DatabasePort + AlertPort` double whose `create_pool` fails with a
/// configurable error a configurable number of times before succeeding.
#[derive(Clone)]
pub struct FlakyDb {
    connect_failures_remaining: std::sync::Arc<std::sync::atomic::AtomicU32>,
    connect_error: DatabaseConnectError,
}

impl FlakyDb {
    /// A pool that fails `create_pool` `failures` times (with `error`) before succeeding.
    pub fn new(failures: u32, error: DatabaseConnectError) -> Self {
        FlakyDb {
            connect_failures_remaining: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(
                failures,
            )),
            connect_error: error,
        }
    }
}

impl DatabasePort for FlakyDb {
    fn new(_conf: &Database) -> Self {
        FlakyDb::new(0, DatabaseConnectError::Retryable("flaky".into()))
    }
    fn create_pool(&mut self) -> impl Future<Output = Result<(), DatabaseConnectError>> + Send {
        let remaining = self
            .connect_failures_remaining
            .load(std::sync::atomic::Ordering::SeqCst);
        let result = if remaining > 0 {
            self.connect_failures_remaining
                .store(remaining - 1, std::sync::atomic::Ordering::SeqCst);
            Err(self.connect_error.clone())
        } else {
            Ok(())
        };
        async move { result }
    }
    fn close_pool(&self) -> impl Future<Output = Result<(), DatabaseCloseError>> + Send {
        async { Ok(()) }
    }
}

impl AdaptorConfigRepr for FlakyDb {
    fn adaptor_name(&self) -> &'static str {
        "flaky_db"
    }
    fn config_fields(&self) -> Vec<AdaptorConfigField> {
        vec![]
    }
}

impl AlertPort for FlakyDb {
    fn get_latest_alerts_data(
        &self,
        _l: u64,
        _o: u64,
    ) -> impl Future<Output = Result<GetLatestAlertsResponse, GetLatestAlertsError>> + Send {
        async move {
            Ok(GetLatestAlertsResponse {
                alerts: vec![],
                total: 42,
            })
        }
    }
    fn get_daily_alerts_data(
        &self,
        _l: u64,
        _o: u64,
    ) -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send {
        async move {
            Ok(GetDailyAlertsResponse {
                alerts: vec![],
                total: 42,
            })
        }
    }
    fn create_alert_data(
        &self,
        alert: Alert,
    ) -> impl Future<Output = Result<CreateAlertResponse, CreateAlertError>> + Send {
        async move { Ok(CreateAlertResponse { alert }) }
    }
    fn update_alert_data(
        &self,
        _identifier: &AlertIdentifier,
        alert: Alert,
    ) -> impl Future<Output = Result<UpdateAlertResponse, UpdateAlertError>> + Send {
        async move { Ok(UpdateAlertResponse { alert }) }
    }
    fn get_alert_data(
        &self,
        identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<GetAlertResponse, GetAlertError>> + Send {
        let identifier = identifier.clone();
        async move {
            Ok(GetAlertResponse {
                alert: mock_alert(identifier),
            })
        }
    }
    fn delete_alert_data(
        &self,
        _identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<DeleteAlertResponse, DeleteAlertError>> + Send {
        async move { Ok(DeleteAlertResponse) }
    }
}

/// `DatabasePort + AlertPort` double that reports a missing alert: reads and
/// deletes for a specific identifier return `NotFound`.
#[derive(Clone)]
pub struct MissingDb;

impl DatabasePort for MissingDb {
    fn new(_conf: &Database) -> Self {
        MissingDb
    }
    fn create_pool(&mut self) -> impl Future<Output = Result<(), DatabaseConnectError>> + Send {
        async { Ok(()) }
    }
    fn close_pool(&self) -> impl Future<Output = Result<(), DatabaseCloseError>> + Send {
        async { Ok(()) }
    }
}

impl AdaptorConfigRepr for MissingDb {
    fn adaptor_name(&self) -> &'static str {
        "missing_db"
    }
    fn config_fields(&self) -> Vec<AdaptorConfigField> {
        vec![]
    }
}

impl AlertPort for MissingDb {
    fn get_latest_alerts_data(
        &self,
        _l: u64,
        _o: u64,
    ) -> impl Future<Output = Result<GetLatestAlertsResponse, GetLatestAlertsError>> + Send {
        async move {
            Ok(GetLatestAlertsResponse {
                alerts: vec![],
                total: 42,
            })
        }
    }
    fn get_daily_alerts_data(
        &self,
        _l: u64,
        _o: u64,
    ) -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send {
        async move {
            Ok(GetDailyAlertsResponse {
                alerts: vec![],
                total: 42,
            })
        }
    }
    fn create_alert_data(
        &self,
        alert: Alert,
    ) -> impl Future<Output = Result<CreateAlertResponse, CreateAlertError>> + Send {
        async move { Ok(CreateAlertResponse { alert }) }
    }
    fn update_alert_data(
        &self,
        _identifier: &AlertIdentifier,
        alert: Alert,
    ) -> impl Future<Output = Result<UpdateAlertResponse, UpdateAlertError>> + Send {
        async move { Ok(UpdateAlertResponse { alert }) }
    }
    fn get_alert_data(
        &self,
        _identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<GetAlertResponse, GetAlertError>> + Send {
        async move { Err(GetAlertError::NotFound) }
    }
    fn delete_alert_data(
        &self,
        _identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<DeleteAlertResponse, DeleteAlertError>> + Send {
        async move { Err(DeleteAlertError::NotFound) }
    }
}

/// Convenience: an `AppState` wired with the supplied `MockMeta` and a real
/// `AlertService` over the supplied `D: DatabasePort + AlertPort` double.
pub fn build_state<D>(meta: MockMeta) -> crate::adaptors::axum::app_state::AppState<MockMeta, D>
where
    D: DatabasePort + AlertPort + Clone,
{
    let conf_service = ConfigService {
        port: MockConfig::new(),
    };
    let db_service = DatabaseService::new(&conf_service);
    let alert_service = AlertService::new(&db_service);
    crate::adaptors::axum::app_state::AppState::new(meta, alert_service)
}

/// Convenience: the router for the alert handlers, bound to `MockMeta`.
pub fn build_alert_app<D>(
    state: crate::adaptors::axum::app_state::AppState<MockMeta, D>,
) -> axum::Router
where
    D: DatabasePort + AlertPort + Clone + Send + Sync + 'static,
{
    use crate::adaptors::axum::handlers::alert::{
        create_alert, delete_alert, get_alerts, get_daily_alerts, patch_alert, update_alert,
    };
    use axum::routing::{delete, get, patch, post, put};

    axum::Router::new()
        .route("/", get(get_alerts::<MockMeta, D>))
        .route("/", post(create_alert::<MockMeta, D>))
        .route("/{identifier}", put(update_alert::<MockMeta, D>))
        .route("/{identifier}", patch(patch_alert::<MockMeta, D>))
        .route("/{identifier}", delete(delete_alert::<MockMeta, D>))
        .route("/daily", get(get_daily_alerts::<MockMeta, D>))
        .with_state(state)
}

/// Convenience: the full router (all route groups) bound to `MockMeta`.
pub fn build_full_app<D>(
    state: crate::adaptors::axum::app_state::AppState<MockMeta, D>,
) -> axum::Router
where
    D: DatabasePort + AlertPort + Clone + Send + Sync + 'static,
{
    crate::adaptors::axum::routes::create_routes::<MockMeta, D>().with_state(state)
}
