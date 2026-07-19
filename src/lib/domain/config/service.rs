use tracing::info;

use crate::domain::config::model::Config;
use crate::domain::config::port::ConfigPort;
use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::logging::port::LoggingPort;
use crate::domain::logging::service::LoggingService;
use crate::domain::webserver::model::Webserver;

#[derive(Debug, Clone)]
pub struct ConfigService<C>
where
    C: ConfigPort,
{
    pub port: C,
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

    /// Get the Config repository
    pub fn get_port(&self) -> &C {
        &self.port
    }

    /// Log the raw configuration inputs.
    pub fn log_raw_config_input(&self) {
        self.port.log_raw_config_input();
    }

    /// Emit the raw configuration through the initialized logging service.
    ///
    /// Must be called after [`LoggingService::new`] so tracing is active.
    /// The `&LoggingService` parameter enforces this ordering at compile time.
    pub fn emit_raw_config<L>(&self, _logging_service: &LoggingService<L>)
    where
        L: LoggingPort,
    {
        let raw = serde_json::to_string_pretty(self.get_raw_config())
            .unwrap_or_else(|_| "failed to serialize config".to_string());
        info!("application configuration:\n{raw}");
    }

    /// Returns the raw configuration.
    pub fn get_raw_config(&self) -> &Config {
        self.port.get_raw_config()
    }

    /// Validate the raw logging configuration.
    pub fn log_raw_config_validation(&self) {
        self.port.log_raw_config_validation();
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
