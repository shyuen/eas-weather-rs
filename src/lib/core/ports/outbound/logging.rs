use crate::core::domain::logging::Logging;

pub trait LoggingRepo {
    fn new(conf: Logging) -> Self;
}
