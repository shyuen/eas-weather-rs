use serde_derive::{Deserialize, Serialize};

use crate::domain::config::model::ConfigLogging;
use crate::domain::logging::new_types::lg_format::LoggingFormat;
use crate::domain::logging::new_types::lg_trace_level::LoggingTraceLevel;
use crate::warn_config_invalid;
use crate::warn_config_not_specified;

/// Configuration for logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn validate_raw_config(&self, raw_log_conf: &ConfigLogging) {
        match &raw_log_conf.format {
            Some(raw_log_format) => {
                if LoggingFormat::new(raw_log_format).is_err() {
                    warn_config_invalid!(
                        "logging.format",
                        raw_log_format,
                        LoggingFormat::default()
                    );
                }
            }
            None => {
                warn_config_not_specified!("logging.format", LoggingFormat::default());
            }
        }

        match &raw_log_conf.trace_level {
            Some(raw_trace_level) => {
                if LoggingTraceLevel::new(raw_trace_level).is_err() {
                    warn_config_invalid!(
                        "logging.trace_level",
                        raw_trace_level,
                        LoggingTraceLevel::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!("logging.trace_level", LoggingTraceLevel::default());
            }
        }
    }
}
