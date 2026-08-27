use dotenvy::dotenv;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use std::env;

use crate::domain::config::issue::ConfigIssue;
use crate::domain::config::model::{Config, RawConfigInputs};
use crate::domain::config::port::ConfigPort;
use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::logging::new_types::lg_format::LoggingFormat;
use crate::domain::logging::new_types::lg_trace_level::LoggingTraceLevel;
use crate::domain::webserver::model::Webserver;

/// The app-name component of the env-var namespace, matching the crate name
/// with hyphens replaced by underscores (`eas-weather-rs` → `EAS_WEATHER_RS`).
/// Used to build the section-scoped prefixes below.
///
/// `concat!` requires a literal (not a const reference), so prefixes are built
/// via the `env_prefix!` macro, which supplies the `APP_NAME` literal and
/// appends a section suffix. figment's `Env` provider can consume these
/// `&'static str` prefixes directly, but clap's `#[arg(env = "...")]` attributes
/// cannot reference a const or macro — they must be written as string literals.
/// The compile-time guard near the bottom keeps those clap literals in sync with
/// the crate name.
macro_rules! env_prefix {
    ($section:literal) => {
        concat!("EAS_WEATHER_RS__", $section)
    };
}

/// Base prefix for all environment variables. Each env var includes the
/// struct section name (e.g. `EAS_WEATHER_RS__WEBSERVER__PORT`) so that
/// after prefix stripping and `__` splitting, figment maps the key into the
/// correct nested `Config` field.
const BASE_ENV_PREFIX: &str = env_prefix!("");

/// Prefix for app-level env vars (top-level `config_file` field).
const APP_SECTION_ENV_PREFIX: &str = env_prefix!("APP__");

/// Const-equality of the crate name and a prefix, treating `-` in the name and
/// `_` in the prefix as skip characters, so `eas-weather-rs` matches
/// `eas_weather_rs`. Free of any allocation, so it is evaluable at compile
/// time.
const fn crate_name_matches(name: &str, prefix: &str) -> bool {
    let name = name.as_bytes();
    let prefix = prefix.as_bytes();
    let (mut i, mut j) = (0, 0);
    while i < name.len() && j < prefix.len() {
        if name[i] == b'-' {
            i += 1;
            continue;
        }
        if prefix[j] == b'_' {
            j += 1;
            continue;
        }
        if name[i] != prefix[j] {
            return false;
        }
        i += 1;
        j += 1;
    }
    // Remaining name bytes must all be `-` (skipped); prefix must be exhausted.
    while i < name.len() {
        if name[i] != b'-' {
            return false;
        }
        i += 1;
    }
    while j < prefix.len() {
        if prefix[j] != b'_' {
            return false;
        }
        j += 1;
    }
    true
}

/// Fail the build if the crate name (underscored) drifts from `APP_NAME`.
///
/// `APP_NAME` cannot be compared to `env!("CARGO_PKG_NAME")` case-insensitively
/// at compile time (no const `to_ascii_lowercase`), so the lowercase form is
/// spelled explicitly here.
const _: () = {
    const APP_NAME_LOWERCASED: &str = "eas_weather_rs";
    assert!(crate_name_matches(
        env!("CARGO_PKG_NAME"),
        APP_NAME_LOWERCASED
    ));
};

#[derive(Debug, Clone)]
pub struct ConfigFigment {
    conf_raw: Config,
    conf_logging: Logging,
    conf_database: Database,
    conf_webserver: Webserver,
    /// Serialized CLI arguments, supplied by the composition root. Carried as a
    /// neutral `serde_json::Value` so this adaptor has no dependency on the clap
    /// adaptor.
    cli: serde_json::Value,
}

/// Build a figment whose first (highest-priority) layer is the serialized CLI
/// arguments, if any. A `null` value contributes no layer.
fn base_figment(cli: &serde_json::Value) -> Figment {
    let mut figment = Figment::new(); // Start with an empty figment
    if !cli.is_null() {
        // Add the serialized CLI arguments as a layer to the figment.
        figment = figment.join(Serialized::defaults(cli.clone()));
    }
    figment
}

/// Collect the raw configuration from all sources (CLI, ENV, FILE) and merge them into a single `Config` struct.
fn collect_raw_input(cli: &serde_json::Value) -> Config {
    // This line loads the environment variables from the ".env" file.
    dotenv().ok();

    // Use Figment to load preliminary configuration
    // We need this to handle the certain configuration options such as using a
    // different configuration file to load when dictated by the CLI or ENV.
    let conf: Config = match base_figment(cli)
        .join(Env::prefixed(APP_SECTION_ENV_PREFIX).split("__"))
        .extract()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error extracting configuration: {}", e);
            std::process::exit(1); // We use exit for planned exits instead of panics
        }
    };

    // Check which selected config file to load from CLI or ENV
    let config_file = match conf.config_file {
        Some(x) => x.to_string(),
        None => "config/config.toml".to_string(),
    };

    // Use Figment to load configuration from multiple sources the following priority
    // CLI > ENV > FILE > DEFAULT FILE > CODE
    let conf: Config = match base_figment(cli)
        .join(Env::prefixed(BASE_ENV_PREFIX).split("__"))
        .join(Toml::file(config_file))
        .join(Toml::file("./config/default.toml"))
        .extract()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error extracting configuration: {}", e);
            std::process::exit(1);
        }
    };

    conf
}

impl ConfigFigment {
    /// Construct the config adaptor from CLI arguments that were already parsed
    /// and serialized by the composition root (see `src/bin/server/main.rs`).
    ///
    /// Passing `serde_json::Value::Null` (or an object) is supported; a `null`
    /// value contributes no CLI layer, which is what [`ConfigPort::new`] uses.
    pub fn with_cli(cli: serde_json::Value) -> Self {
        // Collect the raw configuration from all sources (CLI, ENV, FILE) and merge them into a single `Config` struct.
        let conf = collect_raw_input(&cli);

        // Convert raw logging config to validated domain model
        let log_format = match &conf.logging.format {
            Some(raw_log_format) => {
                LoggingFormat::new(raw_log_format).unwrap_or_else(|_| LoggingFormat::default())
            }
            None => LoggingFormat::default(),
        };
        let log_trace_level = match &conf.logging.trace_level {
            Some(raw_trace_level) => LoggingTraceLevel::new(raw_trace_level)
                .unwrap_or_else(|_| LoggingTraceLevel::default()),
            None => LoggingTraceLevel::default(),
        };

        Self {
            conf_raw: conf.clone(),
            conf_logging: Logging::new(log_format, log_trace_level),
            conf_database: Database::new(&conf.database),
            conf_webserver: Webserver::new(&conf.webserver),
            cli,
        }
    }
}

impl ConfigPort for ConfigFigment {
    fn new() -> Self {
        Self::with_cli(serde_json::Value::Null)
    }

    /// Get the raw configuration
    fn get_raw_config(&self) -> &Config {
        &self.conf_raw
    }

    /// Get the logging configuration
    fn get_logging_config(&self) -> &Logging {
        &self.conf_logging
    }

    /// Get the database configuration
    fn get_database_config(&self) -> &Database {
        &self.conf_database
    }

    /// Get the webserver configuration
    fn get_webserver_config(&self) -> &Webserver {
        &self.conf_webserver
    }

    // Gather raw config input from each source without validation
    fn raw_config_input(&self) -> RawConfigInputs {
        // Gather config from CLI
        let cli = self.cli.clone();

        // Gather config from ENV
        let env = serde_json::to_value(
            env::vars()
                .filter(|(k, _)| k.starts_with(BASE_ENV_PREFIX))
                .map(|(k, v)| (k.replace("__", "."), v))
                .collect::<Vec<_>>(),
        )
        .unwrap_or(serde_json::Value::Null);

        // Gather config from files
        let files: Config = match Figment::new()
            .join(Toml::file(
                self.conf_raw
                    .config_file
                    .clone()
                    .unwrap_or("config/config.toml".to_string()),
            ))
            .join(Toml::file("./config/default.toml"))
            .extract()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error extracting configuration: {}", e);
                std::process::exit(1);
            }
        };
        let files = serde_json::to_value(files).unwrap_or(serde_json::Value::Null);

        // Final merged raw config before validation
        let final_config = serde_json::to_value(&self.conf_raw).unwrap_or(serde_json::Value::Null);

        RawConfigInputs {
            cli,
            env,
            files,
            final_config,
        }
    }

    /// Run validation over the effective configuration, collecting detected issues
    fn validate_raw_config(&self) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Validate logging config
        match &self.conf_raw.logging.format {
            Some(raw_log_format) => {
                if LoggingFormat::new(raw_log_format).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "logging.format",
                        value: raw_log_format.to_string(),
                        default: LoggingFormat::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "logging.format",
                    default: LoggingFormat::default().to_string(),
                });
            }
        }
        match &self.conf_raw.logging.trace_level {
            Some(raw_trace_level) => {
                if LoggingTraceLevel::new(raw_trace_level).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "logging.trace_level",
                        value: raw_trace_level.to_string(),
                        default: LoggingTraceLevel::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "logging.trace_level",
                    default: LoggingTraceLevel::default().to_string(),
                });
            }
        }

        // Validate database config
        issues.extend(
            self.conf_database
                .validate_raw_config(&self.conf_raw.database),
        );

        // Validate webserver config
        issues.extend(
            self.conf_webserver
                .validate_raw_config(&self.conf_raw.webserver),
        );

        issues
    }
}
