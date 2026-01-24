use crate::core::domain::config::database_conn_init_delay_secs::DbConnRetryInitDelaySecs;
use crate::core::domain::config::database_conn_max_retries::DbConnMaxRetries;
use crate::core::domain::config::database_conn_string::{
    DbConnectionString, DbConnectionStringError,
};
use crate::core::domain::config::raw::ConfigDatabase;
use crate::core::ports::outbound::logging::LoggingRepo;

#[derive(Debug)]
pub struct Database {
    pub conn_string: DbConnectionString,
    pub conn_max_retries: DbConnMaxRetries,
    pub conn_retry_init_delay_secs: DbConnRetryInitDelaySecs,
}

impl Database {
    /// Create a new instance of Database configuration
    fn new(conf: &ConfigDatabase) -> Self {
        let conn_string = match &conf.conn_url_file {
            Some(raw_conn_url_file) => {
                DbConnectionString::new(&raw_conn_url_file).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    DbConnectionStringError::BadFileLoad(_) => DbConnectionString::default(),
                    DbConnectionStringError::EmptyConnectionString(_) => {
                        DbConnectionString::default()
                    }
                    DbConnectionStringError::EmptyFilePath(_) => DbConnectionString::default(),
                })
            }
            None => DbConnectionString::default(),
        };

        let conn_max_retries = match &conf.conn_max_retries {
            Some(raw_conn_max_retries) => DbConnMaxRetries::new(raw_conn_max_retries)
                .unwrap_or_else(|err| match &err {
                    _ => {
                        panic!("uncaught DbConnMaxRetriesError");
                    }
                }),
            None => DbConnMaxRetries::default(),
        };

        let conn_retry_init_delay_secs = match &conf.conn_retry_init_delay_secs {
            Some(raw_conn_retry_init_delay_secs) => DbConnRetryInitDelaySecs::new(
                raw_conn_retry_init_delay_secs,
            )
            .unwrap_or_else(|err| match &err {
                _ => {
                    panic!("uncaught DbConnRetryInitDelaySecsError");
                }
            }),
            None => DbConnRetryInitDelaySecs::default(),
        };

        Database {
            conn_string,
            conn_max_retries,
            conn_retry_init_delay_secs,
        }
    }

    pub fn validate_raw_database_config(
        &self,
        log_serv: &impl LoggingRepo,
        raw_db_conf: &ConfigDatabase,
    ) {
        match &raw_db_conf.conn_url_file {
            Some(raw_conn_url_file) => {
                if let Err(err) = DbConnectionString::new(raw_conn_url_file) {
                    match &err {
                        DbConnectionStringError::BadFileLoad(e) => {
                            log_serv.error(module_path!(), &format!("{}", e));

                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "database connection string will be set to `{}`",
                                    DbConnectionString::default()
                                ),
                            );
                        }
                        DbConnectionStringError::EmptyConnectionString(e) => {
                            log_serv.warn(module_path!(), &format!("{}", e));
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "database connection string will be set to `{}`",
                                    DbConnectionString::default()
                                ),
                            );
                        }
                        DbConnectionStringError::EmptyFilePath(e) => {
                            log_serv.warn(module_path!(), &format!("{}", e));
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "database connection string will be set to `{}`",
                                    DbConnectionString::default()
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "database connection string file was not specified, setting database connection string to `{}`",
                        DbConnectionString::default()
                    ),
                );
            }
        };
    }
}
