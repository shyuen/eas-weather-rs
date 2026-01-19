use crate::core::domain::config::logging::Logging;
use crate::core::domain::config::raw::Config;
use crate::core::ports::inbound::config::ConfigRepo;

use crate::core::ports::outbound::logging::LoggingRepo;

#[derive(Debug, Clone)]
pub struct ConfigService<C>
where
    C: ConfigRepo,
{
    pub repo: C,
}

impl<C> ConfigService<C>
where
    C: ConfigRepo,
{
    /// Creates a new instance of ConfigService.
    pub fn new() -> Self {
        let repo = C::new();
        Self { repo }
    }

    /// Log the raw configuration inputs.
    pub fn log_raw_config_input(&self) {
        self.repo.log_raw_config_input();
    }

    /// Returns the raw configuration.
    pub fn get_raw_config(&self) -> &Config {
        self.repo.get_raw_config()
    }

    /// Returns the logging configuration.
    pub fn get_logging_config(&self) -> &Logging {
        self.repo.get_logging_config()
    }

    pub fn validate_logging_config(&self, log_serv: &impl LoggingRepo) {
        self.repo.validate_logging_config(log_serv);
    }
}
