use crate::core::domain::config::logging_format::{LoggingFormat, LoggingFormatError};
use crate::core::domain::config::logging_trace_level::{LoggingTraceLevel, LoggingTraceLevelError};
use crate::core::domain::config::raw::ConfigLogging;
use crate::core::ports::outbound::logging::LoggingRepo;

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

    pub fn validate_raw_logging_config(
        &self,
        log_serv: &impl LoggingRepo,
        raw_log_conf: &ConfigLogging,
    ) {
        match &raw_log_conf.log_format {
            Some(raw_log_format) => {
                if let Err(err) = LoggingFormat::new(&raw_log_format) {
                    match &err {
                        LoggingFormatError::EmptyType(_) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "logging format type is empty, setting to `{}`",
                                    LoggingFormat::default()
                                ),
                            );
                        }
                        LoggingFormatError::UnknownFormat(_) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "unknown logging format `{}`, setting value to `{}`",
                                    raw_log_format,
                                    LoggingFormat::default(),
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "logging format was not specified, setting value to `{}`",
                        LoggingFormat::default()
                    ),
                );
            }
        }

        match &raw_log_conf.log_trace_level {
            Some(raw_trace_level) => {
                if let Err(err) = LoggingTraceLevel::new(&raw_trace_level) {
                    match &err {
                        LoggingTraceLevelError::EmptyTraceLevel(_) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "logging trace level is empty, setting to `{}`",
                                    LoggingTraceLevel::default()
                                ),
                            );
                        }
                        LoggingTraceLevelError::UnknownTraceLevel(_) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "unknown logging trace level `{}`, setting value to `{}`",
                                    raw_trace_level,
                                    LoggingTraceLevel::default(),
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "logging trace level was not specified, setting value to `{}`",
                        LoggingTraceLevel::default()
                    ),
                );
            }
        }
    }
}
