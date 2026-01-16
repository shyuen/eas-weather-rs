use crate::core::domain::logging::Logging;

use crate::core::ports::outbound::logging::LoggingRepo;

pub struct TracingLogging {
    logging: Logging,
}

impl LoggingRepo for TracingLogging {
    fn new(conf: Logging) -> Self {
        TracingLogging { logging: conf }
    }
}
