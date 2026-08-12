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
    pub fn get_port(&self) -> &D {
        &self.db_port
    }

    pub async fn create_pool(&mut self) {
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
                    return;
                }
                Err(DatabaseConnectError::Fatal(msg)) => {
                    error!(attempt, error = %msg, "create_pool: fatal error; aborting");
                    return;
                }
                Err(DatabaseConnectError::Retryable(msg)) => {
                    if attempt == max_retries {
                        error!(
                            attempt,
                            max_retries,
                            error = %msg,
                            "create_pool: connection retries exhausted"
                        );
                        return;
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
