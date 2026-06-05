use crate::domain::database::model::Database;

//#[async_trait]
pub trait DatabasePort: Clone + Send + Sync + 'static {
    /// Create a new instance of the database repository with the given configuration
    fn new(conf: &Database) -> Self;

    /// Log configuration that was set for this service
    fn log_adaptor_config(&self, conf: &Database);

    /// Create the database connection pool
    fn create_pool(&mut self, conf: &Database) -> impl Future<Output = ()> + Send;

    /// Close the database connection pool
    fn close_pool(&self) -> impl Future<Output = ()> + Send;
}
