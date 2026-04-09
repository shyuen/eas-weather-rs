use tracing::{Level, error, info, warn};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::domain::logging::model::Logging;
use crate::domain::logging::new_types::lg_format::LoggingFormatType;
use crate::domain::logging::new_types::lg_trace_level::LoggingTraceLevelType;
use crate::domain::logging::port::LoggingRepo;

#[derive(Debug, Clone)]
pub struct LoggingTracing {}

impl LoggingTracing {
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

impl LoggingRepo for LoggingTracing {
    fn new(conf_log: &Logging) -> Self {
        // Map LoggingTraceLevel to tracing::Level
        let trace_level = Self::map_trace_level(&conf_log.trace_level.get());

        match &conf_log.format.get() {
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
                    .with_ansi(true)
                    .with_max_level(trace_level)
                    .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                    .with_target(false)
                    .init();
            }
        }

        LoggingTracing {}
    }

    /// Log configuration that's currently set
    fn log_adaptor_config(&self, conf_log: &Logging) {
        self.info(
            module_path!(),
            &format!("tracing_format={:?}", conf_log.format.to_string()),
        );
        self.info(
            module_path!(),
            &format!("tracing_trace_level={:?}", conf_log.trace_level.to_string()),
        );
    }

    /// Log an info level message
    fn info(&self, mod_path: &str, message: &str) {
        info!(mod_path, "{}", message);
    }

    /// Log an error level message
    fn error(&self, mod_path: &str, message: &str) {
        error!(mod_path, "{}", message);
    }

    /// Log a debug level message
    fn debug(&self, mod_path: &str, message: &str) {
        tracing::debug!(mod_path, "{}", message);
    }

    /// Log a warn level message
    fn warn(&self, mod_path: &str, message: &str) {
        warn!(mod_path, "{}", message);
    }

    /// Log a trace level message
    fn trace(&self, mod_path: &str, message: &str) {
        tracing::trace!(mod_path, "{}", message);
    }
}
