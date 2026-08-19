use tracing::{debug, warn};

use crate::domain::config::issue::ConfigIssue;
use crate::domain::config::model::{Config, ValidatedConfig};
use crate::domain::config::port::ConfigPort;
use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::logging::new_types::lg_format::LoggingFormatType;
use crate::domain::webserver::model::Webserver;

#[derive(Debug, Clone)]
pub struct ConfigService<CP>
where
    CP: ConfigPort,
{
    pub port: CP,
}

impl<CP> Default for ConfigService<CP>
where
    CP: ConfigPort,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<CP> ConfigService<CP>
where
    CP: ConfigPort,
{
    /// Creates a new instance of ConfigService.
    pub fn new() -> Self {
        let port = CP::new();
        Self { port }
    }

    /// Create a new instance wrapping an already-constructed port.
    ///
    /// Lets the composition root inject adaptor-specific construction inputs
    /// (e.g. pre-parsed CLI arguments) that the parameterless [`ConfigPort::new`]
    /// cannot express.
    pub fn from_config_port(port: CP) -> Self {
        Self { port }
    }

    /// Get the Config port
    pub fn get_config_port(&self) -> &CP {
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

    /// Return the validated application configuration struct
    /// which can be used by a handler
    pub fn get_validated_app_conf(&self) -> ValidatedConfig {
        ValidatedConfig::new(
            self.port.get_logging_config().clone(),
            self.port.get_database_config().clone(),
            self.port.get_webserver_config().clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::MockConfig;

    #[tokio::test]
    async fn get_validated_app_conf_assembles_validated_config_from_port() {
        let conf_serv: ConfigService<MockConfig> = ConfigService::new();
        let conf = conf_serv.get_validated_app_conf();
        assert_eq!(
            serde_json::to_value(conf.get_logging_config()).unwrap(),
            serde_json::to_value(conf_serv.get_logging_config()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(conf.get_database_config()).unwrap(),
            serde_json::to_value(conf_serv.get_database_config()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(conf.get_webserver_config()).unwrap(),
            serde_json::to_value(conf_serv.get_webservicer_config()).unwrap()
        );
    }
}
