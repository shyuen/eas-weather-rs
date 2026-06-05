use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::time::Duration;

use crate::domain::database::model::Database;
use crate::domain::database::port::DatabasePort;

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
                tracing::error!("unable to parse connection string - error: {}", err_msg);
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
                // let db_name = match options.get_database() {
                //     Some(db_name) => &format!("/{}", db_name),
                //     None => "",
                // };

                // log_port.info(
                //     module_path!(),
                //     &format!(
                //         "xsqlx_conn_opt={username}@{host}:{port}{name}",
                //         host = options.get_host(),
                //         port = options.get_port(),
                //         name = db_name,
                //         username = options.get_username(),
                //     ),
                // );

                tracing::info!("xsqlx_conn_opt=\"{}\"", conf_db.conn_string.to_string());
            }
            None => {
                tracing::warn!("database MySQL options were not set successfully");
            }
        }

        tracing::info!("xsqlx_conn_max_retries={}", &conf_db.conn_max_retries);

        tracing::info!(
            "xsqlx_conn_retry_init_delay_secs={}",
            &conf_db.conn_retry_init_delay_secs
        );

        tracing::info!(
            "xsqlx_conn_acquire_timeout_secs={}",
            &conf_db.conn_acquire_timeout_secs
        );

        tracing::info!(
            "xsqlx_conn_idle_timeout_secs={}",
            &conf_db.conn_idle_timeout_secs
        );

        tracing::info!(
            "xsqlx_conn_max_lifetime_secs={}",
            &conf_db.conn_max_lifetime_secs
        );

        tracing::info!("xsqlx_min_connections={}", &conf_db.min_connections);

        tracing::info!("xsqlx_max_connections={}", &conf_db.max_connections);
    }

    async fn create_pool(&mut self, conf_db: &Database) {
        let mut current_backoff = *(&conf_db.conn_retry_init_delay_secs.get());

        match &self.conn_opt {
            Some(conn_opt) => {
                for i in 0..conf_db.conn_max_retries.get() {
                    // Attempt to create the connection pool with configured options
                    match MySqlPoolOptions::new()
                        .acquire_timeout(Duration::from_secs(
                            conf_db.conn_acquire_timeout_secs.get() as u64,
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
                            tracing::info!("database pool created successfully");
                            self.pool = Some(pool);
                            return;
                        }
                        Err(e) => {
                            tracing::warn!(
                                "failed to connect to database (attempt {}/{}): {}",
                                i + 1,
                                conf_db.conn_max_retries.get(),
                                e
                            );
                            if i + 1 == conf_db.conn_max_retries.get() {
                                tracing::error!("database connection retries exhausted all attempts");
                                self.pool = None;
                                return;
                            } else {
                                tracing::warn!(
                                    "database retrying connection in {} seconds",
                                    current_backoff
                                );

                                tokio::time::sleep(std::time::Duration::from_secs(
                                    current_backoff as u64,
                                ))
                                .await;
                                current_backoff *= 2; // Exponential backoff
                            }
                        }
                    }
                }
            }
            None => {
                tracing::error!("database MySQL connection options were not initialized successfully");
            }
        }
    }

    async fn close_pool(&self) {
        match &self.pool {
            Some(pool) => {
                pool.close().await;
                tracing::info!("database connection pool to MySQL closed successfully");
            }
            None => {
                tracing::warn!("database connection pool to MySQL was not initialized");
            }
        }
    }
}
