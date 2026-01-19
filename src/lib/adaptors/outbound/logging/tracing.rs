use tracing::{Level, error, info, warn};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::core::domain::config::logging::Logging;
use crate::core::domain::config::logging_format::LoggingFormatType;
use crate::core::domain::config::logging_trace_level::LoggingTraceLevelType;
use crate::core::ports::outbound::logging::LoggingRepo;

pub struct TracingLogging {}

impl TracingLogging {
    fn map_trace_level(trace_level_type: &LoggingTraceLevelType) -> Level {
        match trace_level_type {
            LoggingTraceLevelType::Error => Level::ERROR,
            LoggingTraceLevelType::Warn => Level::WARN,
            LoggingTraceLevelType::Info => Level::INFO,
            LoggingTraceLevelType::Debug => Level::DEBUG,
            LoggingTraceLevelType::Trace => Level::TRACE,
        }
    }
}

impl LoggingRepo for TracingLogging {
    fn new(conf_log: &Logging) -> Self {
        // Map LoggingTraceLevel to tracing::Level
        let trace_level = Self::map_trace_level(&conf_log.trace_level.trace_level_type());

        match &conf_log.format.format_type() {
            LoggingFormatType::Json => {
                tracing_subscriber::fmt()
                    .json()
                    .with_max_level(trace_level)
                    .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                    .with_target(false)
                    .init();
            }
            LoggingFormatType::Text => {
                tracing_subscriber::fmt()
                    .with_max_level(trace_level)
                    .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                    .with_target(false)
                    .init();
            }
        }

        TracingLogging {}
    }

    /// Log an info level message
    fn info(&self, message: &str) {
        info!("{}", message);
    }

    /// Log an error level message
    fn error(&self, message: &str) {
        error!("{}", message);
    }

    /// Log a debug level message
    fn debug(&self, message: &str) {
        tracing::debug!("{}", message);
    }

    /// Log a warn level message
    fn warn(&self, message: &str) {
        warn!("{}", message);
    }

    /// Log a trace level message
    fn trace(&self, message: &str) {
        tracing::trace!("{}", message);
    }

    /// Log configuration validation messages
    fn log_conf_validation(&self, conf_log: &Logging) {
        self.info(&format!(
            "tracing: logging format set to `{:?}`",
            conf_log.format.format_type()
        ));
        self.info(&format!(
            "tracing: log trace level set to `{:?}`",
            conf_log.trace_level.trace_level_type()
        ));
    }
}
