use strum_macros::EnumString;
use thiserror::Error;

/// A validated and formatted logging trace level.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoggingTraceLevel(LoggingTraceLevelType);

/// Available logging trace level types for application
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub fn new(raw_Logging_trace_level: &str) -> Result<Self, LoggingTraceLevelError> {
        // Add validation logic here if needed
        let trimmed = raw_Logging_trace_level.trim();

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
    pub fn default() -> Self {
        LoggingTraceLevel(LoggingTraceLevelType::Info)
    }
}
