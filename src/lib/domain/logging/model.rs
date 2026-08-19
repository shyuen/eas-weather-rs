use serde_derive::{Deserialize, Serialize};

use crate::domain::logging::new_types::lg_format::LoggingFormat;
use crate::domain::logging::new_types::lg_trace_level::LoggingTraceLevel;

/// Configuration for logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub format: LoggingFormat,
    pub trace_level: LoggingTraceLevel,
}

impl Logging {
    /// Creates a new instance of Logging configuration from validated components.
    pub fn new(format: LoggingFormat, trace_level: LoggingTraceLevel) -> Self {
        Logging {
            format,
            trace_level,
        }
    }
}
