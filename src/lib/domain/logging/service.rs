use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::logging::port::LoggingPort;

#[derive(Debug, Clone)]
pub struct LoggingService<L>
where
    L: LoggingPort,
{
    log_port: L,
}

impl<L> LoggingService<L>
where
    L: LoggingPort,
{
    /// Creates a new instance of LoggingService, installing the logging backend.
    pub fn new<C>(conf_serv: &ConfigService<C>) -> Self
    where
        C: ConfigPort,
    {
        let conf_log = conf_serv.get_logging_config();
        let log_port = L::init(conf_log);
        Self { log_port }
    }

    /// Return a reference to the underlying logging port.
    pub fn get_logging_port(&self) -> &L {
        &self.log_port
    }
}
