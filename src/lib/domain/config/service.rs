use tracing::{debug, info, trace, warn};

use crate::domain::config::adaptor_config::AdaptorConfigRepr;
use crate::domain::config::issue::ConfigIssue;
use crate::domain::config::model::Config;
use crate::domain::config::port::ConfigPort;
use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::logging::new_types::lg_format::LoggingFormatType;
use crate::domain::webserver::model::Webserver;

#[derive(Debug, Clone)]
pub struct ConfigService<C>
where
    C: ConfigPort,
{
    pub port: C,
}

impl<C> Default for ConfigService<C>
where
    C: ConfigPort,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C> ConfigService<C>
where
    C: ConfigPort,
{
    /// Creates a new instance of ConfigService.
    pub fn new() -> Self {
        let port = C::new();
        Self { port }
    }

    /// Create a new instance wrapping an already-constructed port.
    ///
    /// Lets the composition root inject adaptor-specific construction inputs
    /// (e.g. pre-parsed CLI arguments) that the parameterless [`ConfigPort::new`]
    /// cannot express.
    pub fn from_port(port: C) -> Self {
        Self { port }
    }

    /// Get the Config repository
    pub fn get_port(&self) -> &C {
        &self.port
    }

    /// Log the raw configuration inputs gathered from each source.
    ///
    /// Rendering depends on the configured logging format (pretty for text,
    /// compact JSON for the json format). Emitted at `debug` since the raw,
    /// pre-correction inputs are only of interest when troubleshooting config
    /// auto-correction.
    pub fn log_raw_config_input(&self) {
        let inputs = self.port.raw_config_input();
        let pretty = matches!(
            self.get_logging_config().format.get(),
            LoggingFormatType::Text
        );
        let render = |value: &serde_json::Value| -> String {
            if pretty {
                serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
            } else {
                value.to_string()
            }
        };

        debug!(source = "cli", config = %render(&inputs.cli));
        debug!(source = "env", config = %render(&inputs.env));
        debug!(source = "files", config = %render(&inputs.files));
        debug!(source = "final_config", config = %render(&inputs.final_config));
    }

    /// Returns the raw configuration.
    pub fn get_raw_config(&self) -> &Config {
        self.port.get_raw_config()
    }

    /// Log configuration fields exposed by an adaptor through [`AdaptorConfigRepr`].
    ///
    /// Emits one log line per adaptor, so N adaptors produce N lines at startup.
    /// Rendering depends on the configured logging format:
    /// - **text** → the fields are pretty-printed JSON, for convenient developer
    ///   viewing.
    /// - **json** → the fields are emitted as a compact JSON object, the
    ///   machine-readable form suitable for log aggregators such as Splunk.
    ///
    /// Non-sensitive fields are logged at `info`; sensitive fields
    /// (secrets/credentials) are withheld from info and only emitted at `trace`,
    /// and even then in masked form (the adaptor supplies the masked
    /// representation via its `Display`). Real secret values never enter the
    /// logs.
    pub fn log_adaptor_config(&self, adaptor: &impl AdaptorConfigRepr) {
        let adaptor_name = adaptor.adaptor_name();
        let mut public = serde_json::Map::new();
        let mut secrets = serde_json::Map::new();
        for field in adaptor.config_fields() {
            let json = serde_json::Value::String(field.value);
            if field.sensitive {
                secrets.insert(field.key.to_string(), json);
            } else {
                public.insert(field.key.to_string(), json);
            }
        }

        let pretty = matches!(
            self.get_logging_config().format.get(),
            LoggingFormatType::Text
        );

        if !public.is_empty() {
            let config = serde_json::Value::Object(public);
            if pretty {
                info!(
                    adaptor = adaptor_name,
                    config = %serde_json::to_string_pretty(&config).unwrap_or_else(|_| config.to_string())
                );
            } else {
                info!(adaptor = adaptor_name, config = %config);
            }
        }
        if !secrets.is_empty() {
            let config = serde_json::Value::Object(secrets);
            if pretty {
                trace!(
                    adaptor = adaptor_name,
                    config_secret = true,
                    config = %serde_json::to_string_pretty(&config).unwrap_or_else(|_| config.to_string())
                );
            } else {
                trace!(adaptor = adaptor_name, config_secret = true, config = %config);
            }
        }
    }

    /// Validate the raw configuration and log any auto-correction issues.
    ///
    /// Collection happens in the domain models; this service renders each
    /// collected issue as a `warn!` event with structured fields.
    pub fn log_raw_config_validation(&self) {
        for issue in self.port.validate_raw_config() {
            match issue {
                ConfigIssue::NotSpecified { key, default } => {
                    warn!(
                        target: module_path!(),
                        config = key,
                        default = %default,
                        "{} was not specified; defaulting to `{}`",
                        key,
                        default
                    );
                }
                ConfigIssue::Invalid {
                    key,
                    value,
                    default,
                } => {
                    warn!(
                        target: module_path!(),
                        config = key,
                        invalid = %value,
                        default = %default,
                        "{} has invalid value `{}`; defaulting to `{}`",
                        key,
                        value,
                        default
                    );
                }
                ConfigIssue::LoadFailed {
                    key,
                    path,
                    reason,
                    default,
                } => {
                    warn!(
                        target: module_path!(),
                        config = key,
                        path = %path,
                        default = %default,
                        "{} failed to load `{}`: {}; defaulting to `{}`",
                        key,
                        path,
                        reason,
                        default
                    );
                }
            }
        }
    }

    /// Returns the logging configuration.
    pub fn get_logging_config(&self) -> &Logging {
        self.port.get_logging_config()
    }

    /// Returns the database configuration.
    pub fn get_database_config(&self) -> &Database {
        self.port.get_database_config()
    }

    pub fn get_webservicer_config(&self) -> &Webserver {
        self.port.get_webserver_config()
    }
}
