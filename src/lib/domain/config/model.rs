use serde::{Deserialize, Serialize};

/// The raw configuration for the driver and its components.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub logging: ConfigLogging,
    pub webserver: ConfigWebserver,
    pub database: ConfigDatabase,
    pub config_file: Option<String>,
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
