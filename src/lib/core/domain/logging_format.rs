use strum_macros::EnumString;
use thiserror::Error;

/// Logging output format newtype for application
#[derive(Debug)]
pub struct LoggingFormat(LoggingFormatType);

/// Available logging output format types for application
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub fn default() -> Self {
        LoggingFormat(LoggingFormatType::Text)
    }
}
