use thiserror::Error;

use crate::domain::database::model::Database;

/// Errors that can occur while establishing a database connection pool.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DatabaseConnectError {
    /// A transient failure worth retrying (e.g. the DB is temporarily unreachable).
    #[error("database connect failed (retryable): {0}")]
    Retryable(String),
    /// A permanent failure from which retrying will not recover.
    #[error("database connect failed (fatal): {0}")]
    Fatal(String),
}

/// Errors that can occur while closing a database connection pool.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DatabaseCloseError {
    /// The pool was never initialized, so there is nothing to close.
    #[error("database pool was not initialized; nothing to close")]
    PoolNotInitialized,
}

//#[async_trait]
pub trait DatabasePort: Clone + Send + Sync + 'static {
    /// Create a new instance of the database repository with the given configuration
    fn new(conf: &Database) -> Self;

    /// Log configuration that was set for this service
    fn log_adaptor_config(&self, conf: &Database);

    /// Create the database connection pool
    fn create_pool(
        &mut self,
        conf: &Database,
    ) -> impl Future<Output = Result<(), DatabaseConnectError>> + Send;

    /// Close the database connection pool
    fn close_pool(&self) -> impl Future<Output = Result<(), DatabaseCloseError>> + Send;
}
