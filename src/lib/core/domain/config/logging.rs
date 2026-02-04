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
        let format = match &conf.format {
            Some(raw_log_format) => {
                LoggingFormat::new(&raw_log_format).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    _ => LoggingFormat::default(),
                })
            }
            None => LoggingFormat::default(),
        };

        let trace_level = match &conf.trace_level {
            Some(raw_trace_level) => {
                LoggingTraceLevel::new(&raw_trace_level).unwrap_or_else(|err| match &err {
                    // Set value based on raw input or to its default errors
                    // We don't handle logging here as the logger is not yet initialized
                    _ => LoggingTraceLevel::default(),
                })
            }
            None => LoggingTraceLevel::default(),
        };

        Logging {
            format,
            trace_level,
        }
    }

    /// Validates the raw logging configuration and logs warnings for any issues found.
    pub fn validate_raw_logging_config(
        &self,
        log_serv: &impl LoggingRepo,
        raw_log_conf: &ConfigLogging,
    ) {
        match &raw_log_conf.format {
            Some(raw_log_format) => {
                if let Err(err) = LoggingFormat::new(&raw_log_format) {
                    match &err {
                        LoggingFormatError::EmptyType(_) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config logging format type is empty, setting to `{}`",
                                    LoggingFormat::default()
                                ),
                            );
                        }
                        LoggingFormatError::UnknownFormat(_) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config logging format of unknown type `{}`, setting value to `{}`",
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
                        "config logging format was not specified, setting value to `{}`",
                        LoggingFormat::default()
                    ),
                );
            }
        }

        match &raw_log_conf.trace_level {
            Some(raw_trace_level) => {
                if let Err(err) = LoggingTraceLevel::new(&raw_trace_level) {
                    match &err {
                        LoggingTraceLevelError::EmptyTraceLevel(_) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config logging trace level is empty, setting to `{}`",
                                    LoggingTraceLevel::default()
                                ),
                            );
                        }
                        LoggingTraceLevelError::UnknownTraceLevel(_) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config logging trace level of unknown type `{}`, setting value to `{}`",
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
                        "config logging trace level was not specified, setting value to `{}`",
                        LoggingTraceLevel::default()
                    ),
                );
            }
        }
    }
}
