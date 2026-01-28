use crate::core::domain::config::database::Database;
use crate::core::ports::outbound::logging::LoggingRepo;

//#[async_trait]
pub trait DatabaseRepo: Clone + Send + Sync + 'static {
    /// Create a new instance of the database repository with the given configuration
    fn new(log_repo: &impl LoggingRepo, conf: &Database) -> Self;

    /// Log configuration that was set for this service
    fn log_adaptor_config(&self, log_repo: &impl LoggingRepo, conf: &Database);

    /// Create the database connection pool
    fn create_pool(
        &mut self,
        log_repo: &(impl LoggingRepo + Sync),
        conf: &Database,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Close the database connection pool
    fn close_pool(
        &self,
        log_repo: &(impl LoggingRepo + Sync),
    ) -> impl std::future::Future<Output = ()> + Send;
}
