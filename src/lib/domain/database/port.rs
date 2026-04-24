use crate::domain::database::model::Database;
use crate::domain::logging::port::LoggingPort;

//#[async_trait]
pub trait DatabasePort: Clone + Send + Sync + 'static {
    /// Create a new instance of the database repository with the given configuration
    fn new(log_port: &impl LoggingPort, conf: &Database) -> Self;

    /// Log configuration that was set for this service
    fn log_adaptor_config(&self, log_port: &impl LoggingPort, conf: &Database);

    /// Create the database connection pool
    fn create_pool(
        &mut self,
        log_port: &(impl LoggingPort + Sync),
        conf: &Database,
    ) -> impl Future<Output = ()> + Send;

    /// Close the database connection pool
    fn close_pool(&self, log_port: &impl LoggingPort) -> impl Future<Output = ()> + Send;
}
