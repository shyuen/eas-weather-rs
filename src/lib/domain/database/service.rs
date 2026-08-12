use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::{DatabaseConnectError, DatabasePort};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct DatabaseService<D>
where
    D: DatabasePort + AlertPort,
{
    db_port: D,
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
        let conf_db = conf_serv.get_database_config();

        let db_port = D::new(conf_db);
        Self { db_port }
    }

    /// Get the Database port
    pub fn get_port(&self) -> &D {
        &self.db_port
    }

    /// Log configuration that's currently set
    pub fn log_adaptor_config<C>(&self, conf_serv: &ConfigService<C>)
    where
        C: ConfigPort,
    {
        let conf_db = conf_serv.get_database_config();

        self.db_port.log_adaptor_config(conf_db);
    }

    pub async fn create_pool<C>(&mut self, conf_serv: &ConfigService<C>)
    where
        C: ConfigPort,
    {
        let conf_db = conf_serv.get_database_config();
        let max_retries = conf_db.conn_max_retries.get();
        let mut current_backoff = conf_db.conn_retry_init_delay_secs.get();

        debug!(
            "create_pool: creating database connection pool (max_retries={})",
            max_retries
        );

        for attempt in 1..=max_retries {
            match self.db_port.create_pool(conf_db).await {
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
}
