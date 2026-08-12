/// A generic, adaptor-driven representation of an adaptor's configuration.
///
/// Each adaptor implements this for its *own* internal config type so that the
/// config service can log whatever the adaptor dictates — without the adaptor
/// and the config layer sharing a concrete config struct. This keeps adaptors
/// free to diverge (e.g. MySQL vs MongoDB connection settings).
pub trait AdaptorConfigRepr {
    /// Return this adaptor's configuration as (name, value) pairs for logging.
    fn config_fields(&self) -> Vec<(&'static str, String)>;
}
