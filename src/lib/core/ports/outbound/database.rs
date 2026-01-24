use crate::core::domain::config::database::Database;
use crate::core::ports::outbound::logging::LoggingRepo;

pub trait DatabaseRepo {
    /// Create a new instance of the database repository with the given configuration
    fn new(log_repo: &impl LoggingRepo, conf: &Database) -> Self;

    /// Log configuration that was set for this service
    fn log_set_config(&self, log_repo: &impl LoggingRepo, conf: &Database);
}
