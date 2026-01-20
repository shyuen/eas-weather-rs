use crate::core::ports::inbound::config::ConfigRepo;
use crate::core::ports::outbound::logging::LoggingRepo;
use crate::core::services::config::ConfigService;

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
    pub fn new<C>(conf_serv: &ConfigService<C>) -> Self
    where
        C: ConfigRepo,
    {
        let conf_log = conf_serv.get_logging_config();
        let repo = L::new(conf_log);
        Self { repo }
    }

    /// Get the logging repository
    pub fn get_repo(&self) -> &L {
        &self.repo
    }

    /// Log configuration that's currently set
    pub fn log_set_config<C>(&self, conf_serv: &ConfigService<C>)
    where
        C: ConfigRepo,
    {
        let conf_log = conf_serv.get_logging_config();
        self.repo.log_set_config(conf_log);
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
}
