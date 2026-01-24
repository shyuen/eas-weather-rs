use sqlx::mysql::{MySqlConnectOptions, MySqlPool};

use crate::core::domain::config::database::Database;
use crate::core::ports::outbound::database::DatabaseRepo;
use crate::core::ports::outbound::logging::LoggingRepo;

#[derive(Debug, Clone)]
pub struct DatabaseMySql {
    conn_opt: Option<MySqlConnectOptions>,
    pool: Option<sqlx::MySqlPool>, // This is optional to allow application startup without immediate DB connection
}

impl DatabaseMySql {
    pub async fn close_pool(&self, log_repo: &impl LoggingRepo) {
        match &self.pool {
            Some(pool) => {
                pool.close().await;
                log_repo.info(
                    module_path!(),
                    "connection pool to MySQL closed successfully",
                );
            }
            None => {
                log_repo.warn(
                    module_path!(),
                    "connection pool to MySQL was not initialized",
                );
            }
        }
    }

    pub async fn create_pool(&mut self, log_repo: &impl LoggingRepo, conf_db: &Database) {
        let mut current_backoff = *(&conf_db.conn_retry_init_delay_secs.get());

        match &self.conn_opt {
            Some(conn_opt) => {
                for i in 0..conf_db.conn_max_retries.get() {
                    match MySqlPool::connect_with(conn_opt.clone()).await {
                        Ok(pool) => {
                            log_repo.info(module_path!(), "pool created successfully");
                            self.pool = Some(pool);
                            return;
                        }
                        Err(e) => {
                            log_repo.warn(
                                module_path!(),
                                &format!(
                                    "failed to connect to database (attempt {}/{}): {}",
                                    i + 1,
                                    conf_db.conn_max_retries.get(),
                                    e
                                ),
                            );
                            if i + 1 == conf_db.conn_max_retries.get() {
                                log_repo.error(
                                    module_path!(),
                                    "exhausted all retries to connect to database",
                                );
                                self.pool = None;
                                return;
                            } else {
                                log_repo.warn(
                                    module_path!(),
                                    &format!(
                                        "retrying to connect to database in {} seconds",
                                        current_backoff
                                    ),
                                );

                                tokio::time::sleep(std::time::Duration::from_secs(current_backoff))
                                    .await;
                                current_backoff *= 2; // Exponential backoff
                            }
                        }
                    }
                }
            }
            None => {
                log_repo.error(
                    module_path!(),
                    "database MySQL connection options were not initialized successfully",
                );
            }
        }
    }
}

impl DatabaseRepo for DatabaseMySql {
    fn new(log_repo: &impl LoggingRepo, conf_db: &Database) -> Self {
        // Extract connection string
        let conn_string = &conf_db.conn_string.to_string();

        // Attempt to create MySQL connection options
        let conn_opt = match conn_string.parse::<MySqlConnectOptions>() {
            Ok(conn_opt) => Some(conn_opt),
            Err(err_msg) => {
                log_repo.error(
                    module_path!(),
                    &format!("unable to parse connection string - error: {}", err_msg),
                );
                None
            }
        };

        DatabaseMySql {
            conn_opt: conn_opt,
            pool: None,
        }
    }

    /// Log configuration that's currently set
    fn log_set_config(&self, log_repo: &impl LoggingRepo, conf_db: &Database) {
        match &self.conn_opt {
            Some(options) => {
                let dn_name = match options.get_database() {
                    Some(db_name) => &format!("/{}", db_name),
                    None => "",
                };

                log_repo.info(
                    module_path!(),
                    &format!(
                        "database MySQL connection options set to `{host}:{port}{name}` as {username}",
                        host = options.get_host(),
                        port = options.get_port(),
                        name = dn_name,
                        username = options.get_username(),
                    ),
                );
            }
            None => {
                log_repo.warn(
                    module_path!(),
                    "database MySQL options were not set successfully",
                );
            }
        }

        log_repo.info(
            module_path!(),
            &format!("database_conn_max_retries={:?}", &conf_db.conn_max_retries),
        );
        log_repo.info(
            module_path!(),
            &format!(
                "database_conn_retry_init_delay_secs={:?}",
                &conf_db.conn_retry_init_delay_secs
            ),
        );
    }
}
