use crate::domain::logging::model::Logging;

/// Port for the logging backend.
///
/// This port owns only the *configuration* side of logging: installing a
/// backend from config and reporting what was applied. Emission is performed by
/// calling the `tracing` macros directly at each call site, so that the captured
/// target/module-path/file:line reflect the true origin rather than this adaptor.
pub trait LoggingPort: Clone + Send + Sync + 'static {
    /// Configure and install the logging backend from the given configuration.
    fn new(conf: &Logging) -> Self;
}
