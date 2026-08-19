use tracing::{info, trace};

use crate::domain::logging::adaptor_config::AdaptorConfigRepr;
use crate::domain::logging::model::Logging;
use crate::domain::logging::new_types::lg_format::LoggingFormatType;

/// Log configuration fields exposed by an adaptor through [`AdaptorConfigRepr`].
///
/// Emits one log line per adaptor, so N adaptors produce N lines at startup.
/// Rendering depends on the configured logging format:
/// - **text** → the fields are pretty-printed JSON, for convenient developer
///   viewing.
/// - **json** → the fields are emitted as a compact JSON object, the
///   machine-readable form suitable for log aggregators such as Splunk.
///
/// Non-sensitive fields are logged at `info`; sensitive fields
/// (secrets/credentials) are withheld from info and only emitted at `trace`,
/// and even then in masked form (the adaptor supplies the masked
/// representation via its `Display`). Real secret values never enter the
/// logs.
pub fn log_adaptor_config(logging: &Logging, adaptor: &impl AdaptorConfigRepr) {
    let adaptor_name = adaptor.adaptor_name();
    let mut public = serde_json::Map::new();
    let mut secrets = serde_json::Map::new();
    for field in adaptor.config_fields() {
        let json = serde_json::Value::String(field.value);
        if field.sensitive {
            secrets.insert(field.key.to_string(), json);
        } else {
            public.insert(field.key.to_string(), json);
        }
    }

    let pretty = matches!(logging.format.get(), LoggingFormatType::Text);

    if !public.is_empty() {
        let config = serde_json::Value::Object(public);
        if pretty {
            info!(
                adaptor = adaptor_name,
                config = %serde_json::to_string_pretty(&config).unwrap_or_else(|_| config.to_string())
            );
        } else {
            info!(adaptor = adaptor_name, config = %config);
        }
    }
    if !secrets.is_empty() {
        let config = serde_json::Value::Object(secrets);
        if pretty {
            trace!(
                adaptor = adaptor_name,
                config_secret = true,
                config = %serde_json::to_string_pretty(&config).unwrap_or_else(|_| config.to_string())
            );
        } else {
            trace!(adaptor = adaptor_name, config_secret = true, config = %config);
        }
    }
}
