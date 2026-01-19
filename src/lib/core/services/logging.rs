use crate::core::domain::config::logging::Logging;
use crate::core::ports::outbound::logging::LoggingRepo;

#[derive(Debug, Clone)]
pub struct LoggingService<L>
where
    L: LoggingRepo,
{
    pub repo: L,
}

impl<L> LoggingService<L>
where
    L: LoggingRepo,
{
    /// Creates a new instance of LoggingService.
    pub fn new(conf_log: &Logging) -> Self {
        let repo = L::new(conf_log);
        Self { repo }
    }

    pub fn get_repo(&self) -> &L {
        &self.repo
    }

    /// Log an info level message
    pub fn info(&self, message: &str) {
        self.repo.info(message);
    }

    /// Log an error level message
    pub fn error(&self, message: &str) {
        self.repo.error(message);
    }

    /// Log a debug level message
    pub fn debug(&self, message: &str) {
        self.repo.debug(message);
    }

    /// Log a warn level message
    pub fn warn(&self, message: &str) {
        self.repo.warn(message);
    }

    /// Log a trace level message
    pub fn trace(&self, message: &str) {
        self.repo.trace(message);
    }

    pub fn log_conf_validatation(&self, conf_log: &Logging) {
        self.repo.log_conf_validation(conf_log);
    }
}
