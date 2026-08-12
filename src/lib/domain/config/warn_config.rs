//! Standardized auto-correction warnings for configuration validation.
//!
//! These are defined as `#[macro_export]` macros so that `module_path!()`
//! (and thus tracing's target/file/line) resolves to the *call site* module —
//! the config model that detected the problem — rather than to this module.

/// Emit a standardized warning when a configuration setting was not provided and
/// a default value was used in its place during auto-correction.
#[macro_export]
macro_rules! warn_config_not_specified {
    ($config_key:expr, $default:expr $(,)?) => {
        ::tracing::warn!(
            target: module_path!(),
            config = $config_key,
            default = %$default,
            "{} was not specified; defaulting to `{}`",
            $config_key,
            $default
        )
    };
}

/// Emit a standardized warning when a configuration setting had an invalid value
/// and a default value was used in its place during auto-correction.
#[macro_export]
macro_rules! warn_config_invalid {
    ($config_key:expr, $invalid:expr, $default:expr $(,)?) => {
        ::tracing::warn!(
            target: module_path!(),
            config = $config_key,
            invalid = %$invalid,
            default = %$default,
            "{} has invalid value `{}`; defaulting to `{}`",
            $config_key,
            $invalid,
            $default
        )
    };
}

/// Emit a standardized warning when a configuration setting could not be loaded
/// (e.g. reading a secret from a file) and a default value was used instead.
#[macro_export]
macro_rules! warn_config_load_failed {
    ($config_key:expr, $path:expr, $reason:expr, $default:expr $(,)?) => {
        ::tracing::warn!(
            target: module_path!(),
            config = $config_key,
            path = %$path,
            default = %$default,
            "{} failed to load `{}`: {}; defaulting to `{}`",
            $config_key,
            $path,
            $reason,
            $default
        )
    };
}
