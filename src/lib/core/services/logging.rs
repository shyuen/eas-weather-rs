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
    pub fn info(&self, mod_path: &str, message: &str) {
        self.repo.info(mod_path, message);
    }

    /// Log an error level message
    pub fn error(&self, mod_path: &str, message: &str) {
        self.repo.error(mod_path, message);
    }

    /// Log a debug level message
    pub fn debug(&self, mod_path: &str, message: &str) {
        self.repo.debug(mod_path, message);
    }

    /// Log a warn level message
    pub fn warn(&self, mod_path: &str, message: &str) {
        self.repo.warn(mod_path, message);
    }

    /// Log a trace level message
    pub fn trace(&self, mod_path: &str, message: &str) {
        self.repo.trace(mod_path, message);
    }

    pub fn log_conf_validatation(&self, conf_log: &Logging) {
        self.repo.log_conf_validation(conf_log);
    }
}
