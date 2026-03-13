use crate::domain::logging::model::Logging;

pub trait LoggingRepo: Clone + Send + Sync + 'static {
    /// Create a new instance of the logging repository with the given configuration
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

    /// Log configuration that was set for this service
    fn log_adaptor_config(&self, conf: &Logging);
}
