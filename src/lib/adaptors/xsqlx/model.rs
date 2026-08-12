use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::time::Duration;

use crate::domain::config::adaptor_config::{AdaptorConfigField, AdaptorConfigRepr};
use crate::domain::database::model::Database;
use crate::domain::database::new_types::db_conn_string::DbConnectionString;
use crate::domain::database::port::{DatabaseCloseError, DatabaseConnectError, DatabasePort};

#[derive(Debug, Clone)]
pub struct DatabaseMySql {
    conn_string: DbConnectionString,
    conn_max_retries: u8,
    conn_retry_init_delay_secs: u16,
    conn_acquire_timeout_secs: u16,
    conn_idle_timeout_secs: u32,
    conn_max_lifetime_secs: u32,
    min_connections: u32,
    max_connections: u32,
    pool: Option<sqlx::MySqlPool>, // This is optional to allow application startup without immediate DB connection
}

/// Implementation of the DatabaseRepo trait for MySQL using sqlx
impl DatabaseMySql {
    pub fn get_pool(&self) -> Option<&sqlx::MySqlPool> {
        self.pool.as_ref()
    }
}

impl AdaptorConfigRepr for DatabaseMySql {
    fn adaptor_name(&self) -> &'static str {
        "xsqlx"
    }

    fn config_fields(&self) -> Vec<AdaptorConfigField> {
        vec![
            AdaptorConfigField::secret("conn_string", self.conn_string.to_string()),
            AdaptorConfigField::new("conn_max_retries", self.conn_max_retries.to_string()),
            AdaptorConfigField::new(
                "conn_retry_init_delay_secs",
                self.conn_retry_init_delay_secs.to_string(),
            ),
            AdaptorConfigField::new(
                "conn_acquire_timeout_secs",
                self.conn_acquire_timeout_secs.to_string(),
            ),
            AdaptorConfigField::new(
                "conn_idle_timeout_secs",
                self.conn_idle_timeout_secs.to_string(),
            ),
            AdaptorConfigField::new(
                "conn_max_lifetime_secs",
                self.conn_max_lifetime_secs.to_string(),
            ),
            AdaptorConfigField::new("min_connections", self.min_connections.to_string()),
            AdaptorConfigField::new("max_connections", self.max_connections.to_string()),
        ]
    }
}

/// Implementation of the DatabaseRepo trait for MySQL using sqlx
impl DatabasePort for DatabaseMySql {
    fn new(conf_db: &Database) -> Self {
        DatabaseMySql {
            conn_string: conf_db.conn_string.clone(),
            conn_max_retries: conf_db.conn_max_retries.get(),
            conn_retry_init_delay_secs: conf_db.conn_retry_init_delay_secs.get(),
            conn_acquire_timeout_secs: conf_db.conn_acquire_timeout_secs.get(),
            conn_idle_timeout_secs: conf_db.conn_idle_timeout_secs.get(),
            conn_max_lifetime_secs: conf_db.conn_max_lifetime_secs.get(),
            min_connections: conf_db.min_connections.get(),
            max_connections: conf_db.max_connections.get(),
            pool: None,
        }
    }

    async fn create_pool(&mut self) -> Result<(), DatabaseConnectError> {
        let conn_opt = self
            .conn_string
            .get()
            .parse::<MySqlConnectOptions>()
            .map_err(|e| {
                DatabaseConnectError::Fatal(format!("failed to parse connection string: {}", e))
            })?;

        match MySqlPoolOptions::new()
            .acquire_timeout(Duration::from_secs(self.conn_acquire_timeout_secs as u64))
            .idle_timeout(Duration::from_secs(self.conn_idle_timeout_secs as u64))
            .max_lifetime(Duration::from_secs(self.conn_max_lifetime_secs as u64))
            .min_connections(self.min_connections)
            .max_connections(self.max_connections)
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
