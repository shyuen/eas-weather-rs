use serde_derive::{Deserialize, Serialize};

use crate::domain::config::issue::ConfigIssue;
use crate::domain::config::model::ConfigLogging;
use crate::domain::logging::new_types::lg_format::LoggingFormat;
use crate::domain::logging::new_types::lg_trace_level::LoggingTraceLevel;

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
                LoggingFormat::new(raw_log_format).unwrap_or_else(|_| LoggingFormat::default())
            }
            None => LoggingFormat::default(),
        };

        let trace_level = match &conf.trace_level {
            Some(raw_trace_level) => LoggingTraceLevel::new(raw_trace_level)
                .unwrap_or_else(|_| LoggingTraceLevel::default()),
            None => LoggingTraceLevel::default(),
        };

        Logging {
            format,
            trace_level,
        }
    }

    /// Validates the raw logging configuration, collecting any auto-correction
    /// issues. No logging is performed here; the caller renders the issues.
    pub fn validate_raw_config(&self, raw_log_conf: &ConfigLogging) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        match &raw_log_conf.format {
            Some(raw_log_format) => {
                if LoggingFormat::new(raw_log_format).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "logging.format",
                        value: raw_log_format.to_string(),
                        default: LoggingFormat::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "logging.format",
                    default: LoggingFormat::default().to_string(),
                });
            }
        }

        match &raw_log_conf.trace_level {
            Some(raw_trace_level) => {
                if LoggingTraceLevel::new(raw_trace_level).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "logging.trace_level",
                        value: raw_trace_level.to_string(),
                        default: LoggingTraceLevel::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "logging.trace_level",
                    default: LoggingTraceLevel::default().to_string(),
                });
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(format: Option<&str>, trace_level: Option<&str>) -> ConfigLogging {
        ConfigLogging {
            format: format.map(str::to_string),
            trace_level: trace_level.map(str::to_string),
        }
    }

    #[test]
    fn validate_accepts_valid_values() {
        let issues = Logging::new(&raw(Some("json"), Some("error")))
            .validate_raw_config(&raw(Some("json"), Some("error")));
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_flags_invalid_format() {
        let issues =
            Logging::new(&raw(None, None)).validate_raw_config(&raw(Some("xml"), Some("info")));
        assert_eq!(
            issues,
            vec![ConfigIssue::Invalid {
                key: "logging.format",
                value: "xml".into(),
                default: "text".into(),
            }]
        );
    }

    #[test]
    fn validate_flags_invalid_trace_level() {
        let issues =
            Logging::new(&raw(None, None)).validate_raw_config(&raw(Some("text"), Some("noise")));
        assert_eq!(
            issues,
            vec![ConfigIssue::Invalid {
                key: "logging.trace_level",
                value: "noise".into(),
                default: "info".into(),
            }]
        );
    }

    #[test]
    fn validate_flags_missing_format_and_trace_level() {
        let issues = Logging::new(&raw(None, None)).validate_raw_config(&raw(None, None));
        assert_eq!(
            issues,
            vec![
                ConfigIssue::NotSpecified {
                    key: "logging.format",
                    default: "text".into(),
                },
                ConfigIssue::NotSpecified {
                    key: "logging.trace_level",
                    default: "info".into(),
                },
            ]
        );
    }
}
