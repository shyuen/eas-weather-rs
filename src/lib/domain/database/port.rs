use thiserror::Error;

use crate::domain::database::model::Database;
use crate::domain::logging::adaptor_config::AdaptorConfigRepr;

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

/// Errors that can occur while checking database health.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DatabaseHealthError {
    /// The connection pool has not been initialized.
    #[error("database connection pool is not initialized")]
    NotInitialized,
    /// The database is unreachable or returned an error.
    #[error("database is unreachable: {0}")]
    Unreachable(String),
}

//#[async_trait]
pub trait DatabasePort: AdaptorConfigRepr + Clone + Send + Sync + 'static {
    /// Create a new instance of the database repository with the given configuration
    fn new(conf: &Database) -> Self;

    /// Create the database connection pool
    fn create_pool(&mut self) -> impl Future<Output = Result<(), DatabaseConnectError>> + Send;

    /// Close the database connection pool
    fn close_pool(&self) -> impl Future<Output = Result<(), DatabaseCloseError>> + Send;

    /// Check the health of the database connection.
    fn check_health(&self) -> impl Future<Output = Result<(), DatabaseHealthError>> + Send;
}
