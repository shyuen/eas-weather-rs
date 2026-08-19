use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::domain::logging::adaptor_config::{AdaptorConfigField, AdaptorConfigRepr};
use crate::domain::logging::model::Logging;
use crate::domain::logging::new_types::lg_format::LoggingFormat;
use crate::domain::logging::new_types::lg_format::LoggingFormatType;
use crate::domain::logging::new_types::lg_trace_level::LoggingTraceLevel;
use crate::domain::logging::new_types::lg_trace_level::LoggingTraceLevelType;
use crate::domain::logging::port::LoggingPort;

#[derive(Debug, Clone)]
pub struct LoggingTracing {
    format: LoggingFormat,
    trace_level: LoggingTraceLevel,
}

impl LoggingTracing {
    fn map_trace_level(trace_level: &LoggingTraceLevel) -> Level {
        match trace_level.get() {
            LoggingTraceLevelType::Error => Level::ERROR,
            LoggingTraceLevelType::Warn => Level::WARN,
            LoggingTraceLevelType::Info => Level::INFO,
            LoggingTraceLevelType::Debug => Level::DEBUG,
            LoggingTraceLevelType::Trace => Level::TRACE,
        }
    }
}

impl AdaptorConfigRepr for LoggingTracing {
    fn adaptor_name(&self) -> &'static str {
        "tracing"
    }

    fn config_fields(&self) -> Vec<AdaptorConfigField> {
        vec![
            AdaptorConfigField::new("format", self.format.to_string()),
            AdaptorConfigField::new("trace_level", self.trace_level.to_string()),
        ]
    }
}

impl LoggingPort for LoggingTracing {
    fn new(conf_log: &Logging) -> Self {
        let format = conf_log.format.clone();
        let trace_level = conf_log.trace_level.clone();

        // Map LoggingTraceLevel to tracing::Level
        let level = Self::map_trace_level(&trace_level);

        match format.get() {
            LoggingFormatType::Json => {
                tracing_subscriber::fmt()
                    .json()
                    .with_max_level(level)
                    .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                    .with_target(true)
                    .init();
            }
            LoggingFormatType::Text => {
                tracing_subscriber::fmt()
                    .with_ansi(true)
                    .with_max_level(level)
                    .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                    .with_target(true)
                    .init();
            }
        }

        LoggingTracing {
            format,
            trace_level,
        }
    }
}
