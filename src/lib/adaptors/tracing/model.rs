use tracing_subscriber::EnvFilter;
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
    fn trace_level_str(trace_level: &LoggingTraceLevel) -> &'static str {
        match trace_level.get() {
            LoggingTraceLevelType::Error => "error",
            LoggingTraceLevelType::Warn => "warn",
            LoggingTraceLevelType::Info => "info",
            LoggingTraceLevelType::Debug => "debug",
            LoggingTraceLevelType::Trace => "trace",
        }
    }

    /// Build the per-module [`EnvFilter`].
    ///
    /// The configured `trace_level` is applied to the application crate itself
    /// (`eas_weather_rs`), while noisy transitive crates (sqlx, hyper, mio,
    /// tokio, etc.) are pinned to quieter levels. Without this, setting
    /// `trace_level = "debug"` would also dump `debug` output from every
    /// dependency into stdout. The static `RUST_LOG` override still wins when
    /// set, for ad-hoc fine-grained debugging.
    fn build_filter(trace_level: &LoggingTraceLevel) -> EnvFilter {
        let level = Self::trace_level_str(trace_level);
        let directives = format!(
            "eas_weather_rs={level},\
             sqlx=warn,\
             hyper=warn,\
             mio=warn,\
             tokio=info,\
             tower=info,\
             tower_http=info,\
             h2=warn,\
             tonic=warn,\
             rustls=warn,\
             rustls_pki_types=warn,\
             want=warn,\
             reqwest=warn"
        );
        EnvFilter::builder()
            .with_default_directive(directives.parse().expect("static filter is valid"))
            .from_env_lossy()
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
        let filter = Self::build_filter(&trace_level);

        match format.get() {
            LoggingFormatType::Json => {
                tracing_subscriber::fmt()
                    .json()
                    .with_env_filter(filter)
                    .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                    .with_target(true)
                    .init();
            }
            LoggingFormatType::Text => {
                tracing_subscriber::fmt()
                    .with_ansi(true)
                    .with_env_filter(filter)
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
