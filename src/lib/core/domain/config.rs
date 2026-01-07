use serde_derive::{Deserialize, Serialize};

/// The global configuration for the driver and its components.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Config {
    pub(crate) logging: Logging,
    pub(crate) server: Server,
    pub(crate) database: Database,
    pub(crate) config_file: Option<String>,
}

/// The global configuration for logging.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Logging {
    pub(crate) log_format: Option<String>,
    pub(crate) log_trace_level: Option<String>,
}

/// The global configuration for the server and its components.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Server {
    pub(crate) hostname: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) base_path: Option<String>,

    pub(crate) shutdown_timeout_secs: Option<u64>,

    pub(crate) api_key_file: Option<String>,

    pub(crate) jwt_key_file: Option<String>,
    pub(crate) jwt_access_token_expiry_secs: Option<u64>,
}

/// The global configuration for the databse and its components.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Database {
    pub(crate) host: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) name: Option<String>,

    pub(crate) username: Option<String>,
    pub(crate) password_file: Option<String>,

    pub(crate) conn_max_retries: Option<u8>,
    pub(crate) conn_retry_init_delay_secs: Option<u64>,
}
