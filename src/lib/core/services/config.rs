use crate::core::domain::config::database::Database;
use crate::core::domain::config::logging::Logging;
use crate::core::domain::config::raw::Config;
use crate::core::domain::config::webserver::Webserver;
use crate::core::ports::inbound::config::ConfigRepo;
use crate::core::ports::outbound::logging::LoggingRepo;
use crate::core::services::logging::LoggingService;

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
    pub fn log_raw_config_input<L>(&self, logging_serv: &LoggingService<L>)
    where
        L: LoggingRepo,
    {
        let log_repo = logging_serv.get_repo();
        self.repo.log_raw_config_input(log_repo);
    }

    /// Returns the raw configuration.
    pub fn get_raw_config(&self) -> &Config {
        self.repo.get_raw_config()
    }

    /// Validate the raw logging configuration.
    pub fn log_raw_config_validation<L>(&self, logging_serv: &LoggingService<L>)
    where
        L: LoggingRepo,
    {
        let log_repo = logging_serv.get_repo();
        self.repo.log_raw_config_validation(log_repo);
    }

    /// Returns the logging configuration.
    pub fn get_logging_config(&self) -> &Logging {
        self.repo.get_logging_config()
    }

    /// Returns the database configuration.
    pub fn get_database_config(&self) -> &Database {
        self.repo.get_database_config()
    }

    pub fn get_webservicer_config(&self) -> &Webserver {
        self.repo.get_webserver_config()
    }
}
