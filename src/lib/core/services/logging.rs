use crate::core::domain::logging::Logging;
use crate::core::ports::outbound::logging::LoggingRepo;

#[derive(Debug, Clone)]
pub struct LoggingService<L>
where
    L: LoggingRepo,
{
    pub repo: L,
}

impl<L> LoggingService<L>
where
    L: LoggingRepo,
{
    /// Creates a new instance of LoggingService.
    pub fn new(conf: Logging) -> Self {
        let repo = L::new(conf);
        Self { repo: repo }
    }
}
