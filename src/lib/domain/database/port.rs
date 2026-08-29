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

impl DatabaseConnectError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Retryable(_) => "database_connect_retryable",
            Self::Fatal(_) => "database_connect_fatal",
        }
    }
}

/// Errors that can occur while closing a database connection pool.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DatabaseCloseError {
    /// The pool was never initialized, so there is nothing to close.
    #[error("database pool was not initialized; nothing to close")]
    PoolNotInitialized,
}

impl DatabaseCloseError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::PoolNotInitialized => "database_close_pool_not_initialized",
        }
    }
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

impl DatabaseHealthError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotInitialized => "database_health_not_initialized",
            Self::Unreachable(_) => "database_health_unreachable",
        }
    }
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

#[cfg(test)]
mod code_tests {
    use super::*;

    #[test]
    fn database_connect_error_codes() {
        assert_eq!(
            DatabaseConnectError::Retryable("x".into()).code(),
            "database_connect_retryable"
        );
        assert_eq!(
            DatabaseConnectError::Fatal("x".into()).code(),
            "database_connect_fatal"
        );
    }

    #[test]
    fn database_close_error_codes() {
        assert_eq!(
            DatabaseCloseError::PoolNotInitialized.code(),
            "database_close_pool_not_initialized"
        );
    }

    #[test]
    fn database_health_error_codes() {
        assert_eq!(
            DatabaseHealthError::NotInitialized.code(),
            "database_health_not_initialized"
        );
        assert_eq!(
            DatabaseHealthError::Unreachable("x".into()).code(),
            "database_health_unreachable"
        );
    }
}
