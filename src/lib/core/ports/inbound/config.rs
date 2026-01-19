use crate::core::domain::config::logging::Logging;
use crate::core::domain::config::raw::Config;

use crate::core::ports::outbound::logging::LoggingRepo;

pub trait ConfigRepo {
    /// Generate and return the application configuration
    /// Should gather configuration based on the priority order of:
    /// CLI args > Environment File > Environment Variables > Configuration File > Default Configuration File > Default Values
    fn new() -> Self;

    /// Log any validation messages regarding the configuration
    /// This is needed to be triggered after the logging subsystem is initialized
    /// so that configutation log messages can be captured correctly.
    fn log_raw_config_input(&self);

    /// Get the application configuration
    /// Returns a reference to the Config struct
    fn get_raw_config(&self) -> &Config;

    fn get_logging_config(&self) -> &Logging;

    fn validate_logging_config(&self, log_serv: &impl LoggingRepo);
}
