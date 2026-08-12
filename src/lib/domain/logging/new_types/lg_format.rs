use serde_derive::{Deserialize, Serialize};
use std::fmt;
use strum_macros::EnumString;
use thiserror::Error;

/// Logging output format newtype for application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingFormat(LoggingFormatType);

impl fmt::Display for LoggingFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            LoggingFormatType::Text => write!(f, "text"),
            LoggingFormatType::Json => write!(f, "json"),
        }
    }
}

/// Available logging output format types for application
#[derive(
    EnumString, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum LoggingFormatType {
    Text,
    Json,
}

/// Errors related to logging format validation and creation
#[derive(Error, Debug, EnumString)]
pub enum LoggingFormatError {
    #[error("empty log format type was specified")]
    EmptyType(String),
    #[error("unknown log format type was specified")]
    UnknownFormat(String),
}

/// Implementation of LoggingFormatType
impl LoggingFormat {
    pub fn new(raw_logging_format: &str) -> Result<Self, LoggingFormatError> {
        // Add validation logic here if needed
        let trimmed = raw_logging_format.trim();

        if trimmed.trim().is_empty() {
            return Err(LoggingFormatError::EmptyType(trimmed.to_string()));
        }

        match trimmed {
            "text" => Ok(LoggingFormat(LoggingFormatType::Text)),
            "json" => Ok(LoggingFormat(LoggingFormatType::Json)),
            _ => Err(LoggingFormatError::UnknownFormat(trimmed.to_string())),
        }
    }
    pub fn get(&self) -> &LoggingFormatType {
        &self.0
    }
}

impl Default for LoggingFormat {
    fn default() -> Self {
        LoggingFormat(LoggingFormatType::Text)
    }
}
