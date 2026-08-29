use serde::{Deserialize, Serialize};

use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::webserver::model::Webserver;

/// The raw configuration for the driver and its components.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub logging: ConfigLogging,
    pub webserver: ConfigWebserver,
    pub database: ConfigDatabase,
    pub config_file: Option<String>,
}

/// The gathered, pre-validation configuration inputs, split by source.
///
/// Serialized to JSON values so the domain port stays decoupled from the
/// adaptor-specific source types (e.g. the figment `CliServer`). The config service
/// decides how to render them (text vs JSON, masking).
#[derive(Debug, Clone)]
pub struct RawConfigInputs {
    /// Configuration sourced from CLI arguments.
    pub cli: serde_json::Value,
    /// Configuration sourced from environment variables.
    pub env: serde_json::Value,
    /// Configuration sourced from config files.
    pub files: serde_json::Value,
    /// The final merged raw configuration before validation.
    pub final_config: serde_json::Value,
}

/// The raw configuration for logging.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigLogging {
    pub format: Option<String>,
    pub trace_level: Option<String>,
}

/// The raw configuration for the databse and its components.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigDatabase {
    pub conn_url_file: Option<String>,
    pub conn_max_retries: Option<u8>,
    pub conn_retry_init_delay_secs: Option<u16>,

    pub conn_acquire_timeout_secs: Option<u16>,
    pub conn_idle_timeout_secs: Option<u32>,
    pub conn_max_lifetime_secs: Option<u32>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

/// The raw configuration for the server and its components.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigWebserver {
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub base_path: Option<String>,

    pub shutdown_timeout_secs: Option<u64>,

    pub api_key_file: Option<String>,

    pub jwt_key_file: Option<String>,
    pub jwt_access_token_expiry_secs: Option<u64>,

    pub default_page_limit: Option<u64>,
    pub page_limit_max: Option<u64>,
}

/// The validated application configuration, bundled from the validated
/// logging, database, and webserver models. Used by handlers that need the
/// effective configuration as it is applied by the application.
#[derive(Debug, Clone, Serialize)]
pub struct ValidatedConfig {
    conf_logging: Logging,
    conf_database: Database,
    conf_webserver: Webserver,
}

impl ValidatedConfig {
    pub fn new(conf_logging: Logging, conf_database: Database, conf_webserver: Webserver) -> Self {
        Self {
            conf_logging,
            conf_database,
            conf_webserver,
        }
    }

    pub fn get_logging_config(&self) -> &Logging {
        &self.conf_logging
    }

    pub fn get_database_config(&self) -> &Database {
        &self.conf_database
    }

    pub fn get_webserver_config(&self) -> &Webserver {
        &self.conf_webserver
    }
}
