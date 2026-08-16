use crate::domain::config::issue::ConfigIssue;
use crate::domain::config::model::{Config, RawConfigInputs};
use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
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

    /// Gather the raw configuration inputs from each source (CLI, env, files)
    /// without applying validation or auto-correction. Rendering/logging is
    /// left to the caller so it can respect the configured output format.
    fn raw_config_input(&self) -> RawConfigInputs;

    /// Run validation over the effective configuration, collecting the issues
    /// detected during any auto-correction. Emission into the logs is left to
    /// the caller (the config service) so the domain models stay logging-free.
    fn validate_raw_config(&self) -> Vec<ConfigIssue>;
}
