/// A single configuration field exposed by an adaptor for startup logging.
pub struct AdaptorConfigField {
    /// Field name (e.g. `hostname`, `conn_string`).
    pub key: &'static str,
    /// Current effective value.
    pub value: String,
    /// Whether the value is sensitive (secrets/credentials) and must be
    /// withheld from default (info) startup logs.
    pub sensitive: bool,
}

impl AdaptorConfigField {
    /// Build a non-sensitive field.
    pub fn new(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
            sensitive: false,
        }
    }

    /// Build a sensitive field (e.g. api key, jwt key, connection string).
    ///
    /// The supplied `value` should already be a safe-to-log representation
    /// (typically a masked form from the field's `Display`). Sensitive fields
    /// are withheld from `info` logs and only emitted at `trace`, where this
    /// masked value is still used — the real secret never enters the logs.
    pub fn secret(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
            sensitive: true,
        }
    }
}

/// A generic, adaptor-driven representation of an adaptor's configuration.
///
/// Each adaptor implements this for its *own* internal config type so that the
/// config service can log whatever the adaptor dictates — without the adaptor
/// and the config layer sharing a concrete config struct. This keeps adaptors
/// free to diverge (e.g. MySQL vs MongoDB connection settings).
pub trait AdaptorConfigRepr {
    /// Return this adaptor's name, used as a structured log field (e.g. `axum`).
    fn adaptor_name(&self) -> &'static str;

    /// Return this adaptor's configuration as a list of fields for logging.
    fn config_fields(&self) -> Vec<AdaptorConfigField>;
}
