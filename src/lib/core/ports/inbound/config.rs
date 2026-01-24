use crate::core::domain::config::logging::Logging;
use crate::core::domain::config::raw::Config;
use crate::core::ports::outbound::logging::LoggingRepo;

pub trait ConfigRepo {
    /// Generate and return the application configuration
    /// Should gather configuration based on the priority order of:
    /// CLI args > Environment File > Environment Variables > Configuration File > Default Configuration File > Default Values
    fn new() -> Self;

    /// Get the raw configuration
    fn get_raw_config(&self) -> &Config;

    /// Get the logging configurations
    fn get_logging_config(&self) -> &Logging;

    /// Outputs raw config from inputs without validation
    fn log_raw_config_input(&self, log_serv: &impl LoggingRepo);

    /// Log any validation messages regarding the configuration
    /// This is needed to be triggered after the logging subsystem is initialized
    /// so that configutation log messages can be captured correctly.
    fn log_raw_config_validation(&self, log_serv: &impl LoggingRepo);
}
