use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::time::Duration;
use tracing::info;

use crate::domain::database::model::Database;
use crate::domain::database::new_types::db_conn_string::DbConnectionString;
use crate::domain::database::port::{DatabaseCloseError, DatabaseConnectError, DatabasePort};

#[derive(Debug, Clone)]
pub struct DatabaseMySql {
    conn_string: DbConnectionString,
    pool: Option<sqlx::MySqlPool>, // This is optional to allow application startup without immediate DB connection
}

/// Implementation of the DatabaseRepo trait for MySQL using sqlx
impl DatabaseMySql {
    pub fn get_pool(&self) -> Option<&sqlx::MySqlPool> {
        self.pool.as_ref()
    }
}

/// Implementation of the DatabaseRepo trait for MySQL using sqlx
impl DatabasePort for DatabaseMySql {
    fn new(conf_db: &Database) -> Self {
        DatabaseMySql {
            conn_string: conf_db.conn_string.clone(),
            pool: None,
        }
    }

    /// Log configuration that's currently set
    fn log_adaptor_config(&self, conf_db: &Database) {
        info!("xsqlx_conn_opt=\"{}\"", conf_db.conn_string.to_string());

        info!("xsqlx_conn_max_retries={}", &conf_db.conn_max_retries);

        info!(
            "xsqlx_conn_retry_init_delay_secs={}",
            &conf_db.conn_retry_init_delay_secs
        );

        info!(
            "xsqlx_conn_acquire_timeout_secs={}",
            &conf_db.conn_acquire_timeout_secs
        );

        info!(
            "xsqlx_conn_idle_timeout_secs={}",
            &conf_db.conn_idle_timeout_secs
        );

        info!(
            "xsqlx_conn_max_lifetime_secs={}",
            &conf_db.conn_max_lifetime_secs
        );

        info!("xsqlx_min_connections={}", &conf_db.min_connections);

        info!("xsqlx_max_connections={}", &conf_db.max_connections);
    }

    async fn create_pool(&mut self, conf_db: &Database) -> Result<(), DatabaseConnectError> {
        let conn_opt = self
            .conn_string
            .get()
            .parse::<MySqlConnectOptions>()
            .map_err(|e| {
                DatabaseConnectError::Fatal(format!("failed to parse connection string: {}", e))
            })?;

        match MySqlPoolOptions::new()
            .acquire_timeout(Duration::from_secs(
                conf_db.conn_acquire_timeout_secs.get() as u64
            ))
            .idle_timeout(Duration::from_secs(
                conf_db.conn_idle_timeout_secs.get() as u64
            ))
            .max_lifetime(Duration::from_secs(
                conf_db.conn_max_lifetime_secs.get() as u64
            ))
            .min_connections(conf_db.min_connections.get())
            .max_connections(conf_db.max_connections.get())
            .connect_with(conn_opt)
            .await
        {
            Ok(pool) => {
                self.pool = Some(pool);
                Ok(())
            }
            Err(e) => Err(DatabaseConnectError::Retryable(format!(
                "failed to connect to database: {}",
                e
            ))),
        }
    }

    async fn close_pool(&self) -> Result<(), DatabaseCloseError> {
        match &self.pool {
            Some(pool) => {
                pool.close().await;
                Ok(())
            }
            None => Err(DatabaseCloseError::PoolNotInitialized),
        }
    }
}
