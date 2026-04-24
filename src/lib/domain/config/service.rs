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
    pub fn log_raw_config_input<L>(&self, logging_serv: &LoggingService<L>)
    where
        L: LoggingPort,
    {
        let log_port = logging_serv.get_port();
        self.port.log_raw_config_input(log_port);
    }

    /// Returns the raw configuration.
    pub fn get_raw_config(&self) -> &Config {
        self.port.get_raw_config()
    }

    /// Validate the raw logging configuration.
    pub fn log_raw_config_validation<L>(&self, logging_serv: &LoggingService<L>)
    where
        L: LoggingPort,
    {
        let log_port = logging_serv.get_port();
        self.port.log_raw_config_validation(log_port);
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
