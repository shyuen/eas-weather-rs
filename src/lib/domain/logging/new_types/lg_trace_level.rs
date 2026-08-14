use serde_derive::{Deserialize, Serialize};
use std::fmt;
use strum_macros::EnumString;
use thiserror::Error;

/// A validated and formatted logging trace level.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct LoggingTraceLevel(LoggingTraceLevelType);

impl fmt::Display for LoggingTraceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            LoggingTraceLevelType::Error => write!(f, "error"),
            LoggingTraceLevelType::Warn => write!(f, "warn"),
            LoggingTraceLevelType::Info => write!(f, "info"),
            LoggingTraceLevelType::Debug => write!(f, "debug"),
            LoggingTraceLevelType::Trace => write!(f, "trace"),
        }
    }
}

/// Available logging trace level types for application
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, EnumString,
)]
pub enum LoggingTraceLevelType {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Errors related to logging trace level validation and creation
#[derive(Error, Debug, EnumString)]
pub enum LoggingTraceLevelError {
    #[error("LRE-1001: empty trace format type was specified")]
    EmptyTraceLevel(String),
    #[error("LRE-1001: unknown trace format type (`{0}`) was specified")]
    UnknownTraceLevel(String),
}

impl LoggingTraceLevel {
    pub fn new(raw_logging_trace_level: &str) -> Result<Self, LoggingTraceLevelError> {
        // Add validation logic here if needed
        let trimmed = raw_logging_trace_level.trim();

        if trimmed.trim().is_empty() {
            return Err(LoggingTraceLevelError::EmptyTraceLevel(trimmed.to_string()));
        }

        match trimmed {
            "error" => Ok(LoggingTraceLevel(LoggingTraceLevelType::Error)),
            "warn" => Ok(LoggingTraceLevel(LoggingTraceLevelType::Warn)),
            "info" => Ok(LoggingTraceLevel(LoggingTraceLevelType::Info)),
            "debug" => Ok(LoggingTraceLevel(LoggingTraceLevelType::Debug)),
            "trace" => Ok(LoggingTraceLevel(LoggingTraceLevelType::Trace)),
            _ => Err(LoggingTraceLevelError::UnknownTraceLevel(
                trimmed.to_string(),
            )),
        }
    }
    pub fn get(&self) -> &LoggingTraceLevelType {
        &self.0
    }
}

impl Default for LoggingTraceLevel {
    fn default() -> Self {
        LoggingTraceLevel(LoggingTraceLevelType::Info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_valid_levels() {
        assert_eq!(
            LoggingTraceLevel::new("error").unwrap(),
            LoggingTraceLevel(LoggingTraceLevelType::Error)
        );
        assert_eq!(
            LoggingTraceLevel::new("warn").unwrap(),
            LoggingTraceLevel(LoggingTraceLevelType::Warn)
        );
        assert_eq!(
            LoggingTraceLevel::new("info").unwrap(),
            LoggingTraceLevel(LoggingTraceLevelType::Info)
        );
        assert_eq!(
            LoggingTraceLevel::new("debug").unwrap(),
            LoggingTraceLevel(LoggingTraceLevelType::Debug)
        );
        assert_eq!(
            LoggingTraceLevel::new("trace").unwrap(),
            LoggingTraceLevel(LoggingTraceLevelType::Trace)
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            LoggingTraceLevel::new(" debug ").unwrap(),
            LoggingTraceLevel(LoggingTraceLevelType::Debug)
        );
    }

    #[test]
    fn rejects_empty_level() {
        assert!(matches!(
            LoggingTraceLevel::new("").err().unwrap(),
            LoggingTraceLevelError::EmptyTraceLevel(_)
        ));
        assert!(matches!(
            LoggingTraceLevel::new("  ").err().unwrap(),
            LoggingTraceLevelError::EmptyTraceLevel(_)
        ));
    }

    #[test]
    fn rejects_unknown_level() {
        assert!(matches!(
            LoggingTraceLevel::new("verbose").err().unwrap(),
            LoggingTraceLevelError::UnknownTraceLevel(_)
        ));
    }

    #[test]
    fn defaults_to_info() {
        assert_eq!(
            LoggingTraceLevel::default(),
            LoggingTraceLevel(LoggingTraceLevelType::Info)
        );
    }
}
