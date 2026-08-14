//! Configuration validation issues collected by domain models during
//! auto-correction. Models collect and return these; the config service is
//! responsible for rendering/emitting them as log events.

/// A validation issue detected while auto-correcting a raw configuration value.
///
/// Each variant carries the structured data needed to render a distinct
/// logging message — keeping models free of logging while preserving the
/// per-issue `config`, `default`, etc. fields for JSON/Splunk consumption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigIssue {
    /// A configuration setting was not provided and a default was used.
    NotSpecified {
        /// Canonical `sector.field` key of the setting.
        key: &'static str,
        /// The default value used in place of the missing one.
        default: String,
    },
    /// A configuration setting had an invalid value and a default was used.
    Invalid {
        /// Canonical `sector.field` key of the setting.
        key: &'static str,
        /// The offending value that was rejected.
        value: String,
        /// The default value used in place of the invalid one.
        default: String,
    },
    /// A configuration setting failed to load (e.g. reading a secret file).
    LoadFailed {
        /// Canonical `sector.field` key of the setting.
        key: &'static str,
        /// The path/source that failed to load.
        path: String,
        /// The reason the load failed.
        reason: String,
        /// The default value used in place of the failed one.
        default: String,
    },
}
