use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::model::Database;
use crate::domain::database::port::{DatabaseCloseError, DatabaseConnectError, DatabasePort};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct DatabaseService<D>
where
    D: DatabasePort + AlertPort,
{
    db_port: D,
    conf: Database,
}

impl<D> DatabaseService<D>
where
    D: DatabasePort + AlertPort,
{
    /// Creates a new instance of DatabaseService.
    pub fn new<C>(conf_serv: &ConfigService<C>) -> Self
    where
        C: ConfigPort,
    {
        let conf = conf_serv.get_database_config().clone();
        let db_port = D::new(&conf);
        Self { db_port, conf }
    }

    /// Get the Database port
    pub fn get_database_port(&self) -> &D {
        &self.db_port
    }

    pub async fn create_pool(&mut self) -> Result<(), DatabaseConnectError> {
        let max_retries = self.conf.conn_max_retries.get();
        let mut current_backoff = self.conf.conn_retry_init_delay_secs.get();

        debug!(
            "create_pool: creating database connection pool (max_retries={})",
            max_retries
        );

        for attempt in 1..=max_retries {
            match self.db_port.create_pool().await {
                Ok(()) => {
                    info!(
                        attempt,
                        "create_pool: database connection pool created successfully"
                    );
                    return Ok(());
                }
                Err(DatabaseConnectError::Fatal(msg)) => {
                    error!(attempt, error = %msg, "create_pool: fatal error; aborting");
                    return Err(DatabaseConnectError::Fatal(msg));
                }
                Err(DatabaseConnectError::Retryable(msg)) => {
                    if attempt == max_retries {
                        error!(
                            attempt,
                            max_retries,
                            error = %msg,
                            "create_pool: connection retries exhausted"
                        );
                        return Err(DatabaseConnectError::Retryable(msg));
                    }
                    warn!(
                        attempt,
                        max_retries,
                        backoff_secs = current_backoff,
                        error = %msg,
                        "create_pool: connection failed; retrying"
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(current_backoff as u64))
                        .await;
                    current_backoff *= 2;
                }
            }
        }

        // max_retries == 0 means no connection attempt is made.
        Err(DatabaseConnectError::Retryable(
            "create_pool: conn_max_retries is 0; no connection attempted".to_string(),
        ))
    }

    /// Close the database connection pool.
    pub async fn close_pool(&self) -> Result<(), DatabaseCloseError> {
        debug!("close_pool: closing database connection pool");
        match self.db_port.close_pool().await {
            Ok(()) => {
                info!("close_pool: database connection pool closed successfully");
                Ok(())
            }
            Err(err) => {
                warn!("close_pool: {}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::database::new_types::db_conn_acquire_timeout_secs::DbConnAcquireTimeoutSecs;
    use crate::domain::database::new_types::db_conn_idle_timeout_secs::DbConnIdleTimeoutSecs;
    use crate::domain::database::new_types::db_conn_init_delay_secs::DbConnRetryInitDelaySecs;
    use crate::domain::database::new_types::db_conn_max_lifetime_secs::DbConnMaxLifetimeSecs;
    use crate::domain::database::new_types::db_conn_max_retries::DbConnMaxRetries;
    use crate::domain::database::new_types::db_conn_string::DbConnectionString;
    use crate::domain::database::new_types::db_max_connections::DbMaxConnections;
    use crate::domain::database::new_types::db_min_connections::DbMinConnections;
    use crate::test_support::{FlakyDb, MockDb};

    fn db_conf(max_retries: u8, retry_delay_secs: u16) -> Database {
        Database {
            conn_string: DbConnectionString::default(),
            conn_max_retries: DbConnMaxRetries::new(&max_retries).unwrap(),
            conn_retry_init_delay_secs: DbConnRetryInitDelaySecs::new(&retry_delay_secs).unwrap(),
            conn_acquire_timeout_secs: DbConnAcquireTimeoutSecs::new(&1).unwrap(),
            conn_idle_timeout_secs: DbConnIdleTimeoutSecs::new(&1).unwrap(),
            conn_max_lifetime_secs: DbConnMaxLifetimeSecs::new(&1).unwrap(),
            max_connections: DbMaxConnections::new(&1).unwrap(),
            min_connections: DbMinConnections::new(&1).unwrap(),
        }
    }

    #[tokio::test]
    async fn create_pool_succeeds_on_first_attempt() {
        let mut service = DatabaseService {
            db_port: FlakyDb::new(0, DatabaseConnectError::Retryable("x".into())),
            conf: db_conf(3, 0),
        };
        assert!(service.create_pool().await.is_ok());
    }

    #[tokio::test]
    async fn create_pool_retries_then_succeeds() {
        let mut service = DatabaseService {
            db_port: FlakyDb::new(2, DatabaseConnectError::Retryable("x".into())),
            conf: db_conf(3, 0),
        };
        assert!(service.create_pool().await.is_ok());
    }

    #[tokio::test]
    async fn create_pool_returns_error_when_retries_exhausted() {
        let mut service = DatabaseService {
            db_port: FlakyDb::new(5, DatabaseConnectError::Retryable("x".into())),
            conf: db_conf(3, 0),
        };
        match service.create_pool().await {
            Err(DatabaseConnectError::Retryable(_)) => {}
            other => panic!("expected Retryable error, got: {:?}", other.map(|_| ())),
        }
    }

    #[tokio::test]
    async fn create_pool_fatal_error_aborts_immediately() {
        let mut service = DatabaseService {
            db_port: FlakyDb::new(5, DatabaseConnectError::Fatal("boom".into())),
            conf: db_conf(3, 0),
        };
        match service.create_pool().await {
            Err(DatabaseConnectError::Fatal(msg)) => assert_eq!(msg, "boom"),
            other => panic!("expected Fatal error, got: {:?}", other.map(|_| ())),
        }
    }

    #[tokio::test]
    async fn create_pool_with_max_retries_zero_never_attempts() {
        let mut service = DatabaseService {
            db_port: FlakyDb::new(0, DatabaseConnectError::Retryable("x".into())),
            conf: db_conf(0, 0),
        };
        match service.create_pool().await {
            Err(DatabaseConnectError::Retryable(msg)) => {
                assert!(msg.contains("conn_max_retries is 0"))
            }
            other => panic!("expected Retryable error, got: {:?}", other.map(|_| ())),
        }
    }

    #[tokio::test]
    async fn close_pool_returns_ok() {
        let mut service = DatabaseService {
            db_port: MockDb,
            conf: db_conf(3, 0),
        };
        service.create_pool().await.unwrap();
        assert!(service.close_pool().await.is_ok());
    }
}
