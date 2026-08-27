use clap::Parser;
use serde_derive::Serialize;

/// CLI Configuration for the application. Please refer to `config/default.toml` for default values.
/// Should closely follow the structure of the `Config` struct in `crate::domain::config::model`.
#[derive(Debug, Parser, Serialize)]
#[command(version)]
pub(crate) struct Cli {
    #[clap(flatten)]
    logging: Logging,
    #[clap(flatten)]
    webserver: Webserver,
    #[clap(flatten)]
    database: Database,

    /// Config file path location to be used by the server.
    #[arg(short = 'c', long, env = "EAS_WEATHER_RS__APP__CONFIG_FILE")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) config_file: Option<String>,
}

/// Parse the process command-line arguments and serialize them to a neutral
/// JSON value for the config adaptor.
///
/// Called once by the composition root (`src/bin/server/main.rs`); the figment
/// adaptor consumes the serialized value, so the two adaptors never reference
/// each other.
pub fn parse_cli() -> serde_json::Value {
    serde_json::to_value(Cli::parse()).unwrap_or(serde_json::Value::Null)
}

#[derive(Debug, Parser, Serialize)]
struct Logging {
    /// Log format to be used by the server
    #[arg(short = 'l', long, env = "EAS_WEATHER_RS__LOGGING__FORMAT")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) format: Option<String>,

    /// Log trace level to be used by the server
    #[arg(short = 't', long, env = "EAS_WEATHER_RS__LOGGING__TRACE_LEVEL")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) trace_level: Option<String>,
}

#[derive(Debug, Parser, Serialize)]
struct Webserver {
    /// Host name for the server
    #[arg(
        short = 'n',
        long = "hostname",
        env = "EAS_WEATHER_RS__WEBSERVER__HOSTNAME"
    )]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) hostname: Option<String>,

    /// Port number to be used by the server
    #[arg(short = 'p', long, env = "EAS_WEATHER_RS__WEBSERVER__PORT")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) port: Option<u16>,

    /// Subdir path to be used by the server
    #[arg(short = 'b', long, env = "EAS_WEATHER_RS__WEBSERVER__BASE_PATH")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) base_path: Option<String>,

    /// Graceful shutdown timeout in seconds
    #[arg(long, env = "EAS_WEATHER_RS__WEBSERVER__SHUTDOWN_TIMEOUT_SECS")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) shutdown_timeout_secs: Option<u64>,

    /// Path to the API key file
    #[arg(short = 'k', long, env = "EAS_WEATHER_RS__WEBSERVER__API_KEY_FILE")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) api_key_file: Option<String>,

    /// Path to the JWT key file
    #[arg(short = 'j', long, env = "EAS_WEATHER_RS__WEBSERVER__JWT_KEY_FILE")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) jwt_key_file: Option<String>,

    /// JWT access token expiry in seconds
    #[arg(long, env = "EAS_WEATHER_RS__WEBSERVER__JWT_ACCESS_TOKEN_EXPIRY_SECS")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) jwt_access_token_expiry_secs: Option<u64>,

    /// Default page limit for paginated endpoints
    #[arg(long, env = "EAS_WEATHER_RS__WEBSERVER__DEFAULT_PAGE_LIMIT")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) default_page_limit: Option<u64>,

    /// Maximum page limit for paginated endpoints
    #[arg(long, env = "EAS_WEATHER_RS__WEBSERVER__PAGE_LIMIT_MAX")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) page_limit_max: Option<u64>,
}

#[derive(Debug, Parser, Serialize)]
struct Database {
    /// Database connection string file path
    #[arg(short = 'D', long, env = "EAS_WEATHER_RS__DATABASE__CONN_URL_FILE")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_url_file: Option<String>,

    /// Maximum number of database connection retries
    #[arg(short = 'R', long, env = "EAS_WEATHER_RS__DATABASE__CONN_MAX_RETRIES")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_max_retries: Option<u8>,

    /// Initial delay in seconds before retrying database connection
    #[arg(
        short = 'I',
        long,
        env = "EAS_WEATHER_RS__DATABASE__CONN_RETRY_INIT_DELAY_SECS"
    )]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_retry_init_delay_secs: Option<u16>,

    /// Maximum number of database connections
    #[arg(short = 'M', long, env = "EAS_WEATHER_RS__DATABASE__MAX_CONNECTIONS")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) max_connections: Option<u32>,

    /// Minimum number of database connections
    #[arg(short = 'm', long, env = "EAS_WEATHER_RS__DATABASE__MIN_CONNECTIONS")]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) min_connections: Option<u32>,

    /// Database connection acquire timeout in seconds
    #[arg(
        short = 'A',
        long,
        env = "EAS_WEATHER_RS__DATABASE__CONN_ACQUIRE_TIMEOUT_SECS"
    )]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_acquire_timeout_secs: Option<u16>,

    /// Database connection idle timeout in seconds
    #[arg(
        short = 'E',
        long,
        env = "EAS_WEATHER_RS__DATABASE__CONN_IDLE_TIMEOUT_SECS"
    )]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_idle_timeout_secs: Option<u32>,

    /// Database connection maximum lifetime in seconds
    #[arg(
        short = 'L',
        long,
        env = "EAS_WEATHER_RS__DATABASE__CONN_MAX_LIFETIME_SECS"
    )]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub(crate) conn_max_lifetime_secs: Option<u32>,
}
