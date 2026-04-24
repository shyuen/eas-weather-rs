use crate::domain::config::model::Config;
use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::logging::port::LoggingPort;
use crate::domain::webserver::model::Webserver;

pub trait ConfigPort: Clone + Send + Sync + 'static {
    /// Generate and return the application configuration
    /// Should gather configuration based on the priority order of:
    /// CLI args > Environment File > Environment Variables > Configuration File > Default Configuration File > Default Values
    fn new() -> Self;

    /// Get the raw configuration
    fn get_raw_config(&self) -> &Config;

    /// Get the logging configuration
    fn get_logging_config(&self) -> &Logging;

    /// Get the database configuration
    fn get_database_config(&self) -> &Database;

    /// Get the webserver configuration
    fn get_webserver_config(&self) -> &Webserver;

    /// Outputs raw config from inputs without validation to stdout
    fn log_raw_config_input(&self, log_serv: &impl LoggingPort);

    /// Log any validation messages regarding the configuration to stdout
    /// This is needed to be triggered after the logging subsystem is initialized
    /// so that configutation log messages can be captured correctly.
    fn log_raw_config_validation(&self, log_serv: &impl LoggingPort);
}
