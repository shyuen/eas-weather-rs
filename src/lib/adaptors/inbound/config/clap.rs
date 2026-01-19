use clap::Parser;
use serde_derive::Serialize;

/// CLI Configuration for the application. Please refer to `config/default.toml` for default values.
/// Should closly follow the structure of the `Config` struct in `core::domain::config::raw`.
#[derive(Debug, Parser, Serialize)]
#[command(version)]
pub(crate) struct Cli {
    #[clap(flatten)]
    logging: Logging,
    #[clap(flatten)]
    server: Server,
    #[clap(flatten)]
    database: Database,

    /// Config file path location to be used by the server.
    #[arg(short = 'c', long, env = "APP__CONFIG_FILE")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) config_file: Option<String>,
}

#[derive(Debug, Parser, Serialize)]
struct Logging {
    /// Log format to be used by the server
    #[arg(short = 'l', long, env = "APP__LOG_FORMAT")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) format: Option<String>,

    /// Log trace level to be used by the server
    #[arg(short = 't', long, env = "APP__LOG_TRACE_LEVEL")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) trace_level: Option<String>,
}

#[derive(Debug, Parser, Serialize)]
struct Server {
    /// Host name for the server
    #[arg(short = 'n', long = "hostname", env = "SERVER__HOSTNAME")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) hostname: Option<String>,

    /// Port number to be used by the server
    #[arg(short = 'p', long, env = "SERVER__PORT")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) port: Option<u16>,

    /// Subdir path to be used by the server
    #[arg(short = 'b', long, env = "SERVER__BASE_PATH")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) base_path: Option<String>,

    /// Path to the API key file
    #[arg(short = 'k', long, env = "APP__API_KEY_FILE")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) api_key_file: Option<String>,

    /// Path to the JWK key file
    #[arg(short = 'j', long, env = "APP__JWK_KEY_FILE")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) jwk_key_file: Option<String>,
}

#[derive(Debug, Parser, Serialize)]
struct Database {
    /// Database connection string file path
    #[arg(short = 'D', long, env = "DATABASE__CONN_URL_FILE")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_url_file: Option<String>,

    /// Maximum number of database connection retries
    #[arg(short = 'R', long, env = "DATABASE__CONN_MAX_RETRIES")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_max_retries: Option<u8>,

    /// Initial delay in seconds before retrying database connection
    #[arg(short = 'I', long, env = "DATABASE__CONN_RETRY_INIT_DELAY_SECS")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_retry_init_delay_secs: Option<u64>,
}
