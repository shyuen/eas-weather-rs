use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::time::Duration;
use tracing::{error, info, warn};

use crate::domain::database::model::Database;
use crate::domain::database::port::{DatabaseConnectError, DatabasePort};

#[derive(Debug, Clone)]
pub struct DatabaseMySql {
    conn_opt: Option<MySqlConnectOptions>,
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
        // Extract connection string
        //let conn_string = &conf_db.conn_string.to_string();

        let conn_string = &conf_db.conn_string.get();

        // Attempt to create MySQL connection options
        let conn_opt = match conn_string.parse::<MySqlConnectOptions>() {
            Ok(conn_opt) => Some(conn_opt),
            Err(err_msg) => {
                error!("unable to parse connection string - error: {}", err_msg);
                None
            }
        };

        DatabaseMySql {
            conn_opt: conn_opt,
            pool: None,
        }
    }

    /// Log configuration that's currently set
    fn log_adaptor_config(&self, conf_db: &Database) {
        match &self.conn_opt {
            Some(_) => {
                info!("xsqlx_conn_opt=\"{}\"", conf_db.conn_string.to_string());
            }
            None => {
                warn!("database MySQL options were not set successfully");
            }
        }

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
        match &self.conn_opt {
            Some(conn_opt) => {
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
                    .connect_with(conn_opt.clone())
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
            None => Err(DatabaseConnectError::Fatal(
                "database MySQL connection options were not initialized successfully".to_string(),
            )),
        }
    }

    async fn close_pool(&self) {
        match &self.pool {
            Some(pool) => {
                pool.close().await;
                info!("database connection pool to MySQL closed successfully");
            }
            None => {
                warn!("database connection pool to MySQL was not initialized");
            }
        }
    }
}
