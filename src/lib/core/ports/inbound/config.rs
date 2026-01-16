use crate::core::domain::config::Config;

pub trait ConfigRepo {
    /// Generate and return the application configuration
    /// Should gather configuration based on the priority order of:
    /// CLI args > Environment File > Environment Variables > Configuration File > Default Configuration File > Default Values
    fn new() -> Self;

    /// Log any validation messages regarding the configuration
    /// This is needed to be triggered after the logging subsystem is initialized
    /// so that configutation log messages can be captured correctly.
    fn log_config_validation(&self);

    /// Get the application configuration
    /// Returns a reference to the Config struct
    fn get_config(&self) -> &Config;
}
