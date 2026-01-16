use crate::core::domain::logging_format::{LoggingFormat, LoggingFormatError};
use crate::core::domain::logging_trace_level::{LoggingTraceLevel, LoggingTraceLevelError};

use crate::core::domain::config::ConfigLogging;

/// Configuration for logging.
#[derive(Debug)]
pub struct Logging {
    pub format: LoggingFormat,
    pub trace_level: LoggingTraceLevel,
}

impl Logging {
    /// Creates a new instance of Logging configuration.
    pub fn new(conf: &ConfigLogging) -> Self {
        let format = match &conf.log_format {
            // Set value based on raw input or to its default errors
            // We don't handle logging here as the logger is not yet initialized
            Some(raw_log_format) => {
                LoggingFormat::new(&raw_log_format).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    LoggingFormatError::EmptyType(_) => LoggingFormat::default(),
                    LoggingFormatError::UnknownFormat(_) => LoggingFormat::default(),
                })
            }
            None => LoggingFormat::default(),
        };

        let trace_level = match &conf.log_trace_level {
            // Set value based on raw input or to its default errors
            // We don't handle logging here as the logger is not yet initialized
            Some(raw_trace_level) => {
                LoggingTraceLevel::new(&raw_trace_level).unwrap_or_else(|err| match &err {
                    LoggingTraceLevelError::EmptyTraceLevel(_) => LoggingTraceLevel::default(),
                    LoggingTraceLevelError::UnknownTraceLevel(_) => LoggingTraceLevel::default(),
                })
            }
            None => LoggingTraceLevel::default(),
        };

        Logging {
            format,
            trace_level,
        }
    }
}
