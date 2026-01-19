use crate::core::domain::config::logging::Logging;

pub trait LoggingRepo {
    fn new(conf: &Logging) -> Self;

    /// Log an info level message
    fn info(&self, target: &str, message: &str);

    /// Log an error level message
    fn error(&self, target: &str, message: &str);

    /// Log a debug level message
    fn debug(&self, target: &str, message: &str);

    /// Log a warn level message
    fn warn(&self, target: &str, message: &str);

    /// Log a trace level message
    fn trace(&self, target: &str, message: &str);

    fn log_conf_validation(&self, conf: &Logging);
}
