use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::logging::port::LoggingPort;

#[derive(Debug, Clone)]
pub struct LoggingService<L>
where
    L: LoggingPort,
{
    port: L,
}

impl<L> LoggingService<L>
where
    L: LoggingPort,
{
    /// Creates a new instance of LoggingService.
    pub fn new<C>(conf_serv: &ConfigService<C>) -> Self
    where
        C: ConfigPort,
    {
        let conf_log = conf_serv.get_logging_config();
        let port = L::new(conf_log);
        Self { port }
    }

    /// Get the logging portsitory
    pub fn get_port(&self) -> &L {
        &self.port
    }

    /// Log configuration that's currently set
    pub fn log_adaptor_config<C>(&self, conf_serv: &ConfigService<C>)
    where
        C: ConfigPort,
    {
        let conf_log = conf_serv.get_logging_config();
        self.port.log_adaptor_config(conf_log);
    }

    /// Log an info level message
    pub fn info(&self, mod_path: &str, message: &str) {
        self.port.info(mod_path, message);
    }

    /// Log an error level message
    pub fn error(&self, mod_path: &str, message: &str) {
        self.port.error(mod_path, message);
    }

    /// Log a debug level message
    pub fn debug(&self, mod_path: &str, message: &str) {
        self.port.debug(mod_path, message);
    }

    /// Log a warn level message
    pub fn warn(&self, mod_path: &str, message: &str) {
        self.port.warn(mod_path, message);
    }

    /// Log a trace level message
    pub fn trace(&self, mod_path: &str, message: &str) {
        self.port.trace(mod_path, message);
    }
}
