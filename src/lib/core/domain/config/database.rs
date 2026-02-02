use crate::core::domain::config::database_conn_acquire_timeout_secs::DbConnAcquireTimeoutSecs;
use crate::core::domain::config::database_conn_idle_timeout_secs::DbConnIdleTimeoutSecs;
use crate::core::domain::config::database_conn_init_delay_secs::DbConnRetryInitDelaySecs;
use crate::core::domain::config::database_conn_max_lifetime_secs::DbConnMaxLifetimeSecs;
use crate::core::domain::config::database_conn_max_retries::DbConnMaxRetries;
use crate::core::domain::config::database_conn_string::{
    DbConnectionString, DbConnectionStringError,
};
use crate::core::domain::config::database_max_connections::DbMaxConnections;
use crate::core::domain::config::database_min_connections::DbMinConnections;
use crate::core::domain::config::raw::ConfigDatabase;
use crate::core::ports::outbound::logging::LoggingRepo;

#[derive(Debug)]
pub struct Database {
    pub conn_string: DbConnectionString,
    pub conn_max_retries: DbConnMaxRetries,
    pub conn_retry_init_delay_secs: DbConnRetryInitDelaySecs,

    pub conn_acquire_timeout_secs: DbConnAcquireTimeoutSecs,
    pub conn_idle_timeout_secs: DbConnIdleTimeoutSecs,
    pub conn_max_lifetime_secs: DbConnMaxLifetimeSecs,
    pub max_connections: DbMaxConnections,
    pub min_connections: DbMinConnections,
}

impl Database {
    /// Create a new instance of Database configuration
    pub fn new(conf: &ConfigDatabase) -> Self {
        let conn_string = match &conf.conn_url_file {
            Some(raw_conn_url_file) => {
                DbConnectionString::new(&raw_conn_url_file).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    _ => DbConnectionString::default(),
                })
            }
            None => DbConnectionString::default(),
        };

        let conn_max_retries = match &conf.conn_max_retries {
            Some(raw_conn_max_retries) => DbConnMaxRetries::new(raw_conn_max_retries)
                .unwrap_or_else(|err| match &err {
                    _ => {
                        eprintln!("uncaught DbConnMaxRetriesError");
                        std::process::exit(1); // We use exit for planned exits instead of panics
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
                    eprintln!("uncaught DbConnRetryInitDelaySecsError");
                    std::process::exit(1); // We use exit for planned exits instead of panics
                }
            }),
            None => DbConnRetryInitDelaySecs::default(),
        };

        let conn_acquire_timeout_secs = match &conf.conn_acquire_timeout_secs {
            Some(raw_conn_acquire_timeout_secs) => {
                DbConnAcquireTimeoutSecs::new(raw_conn_acquire_timeout_secs).unwrap_or_else(|err| {
                    match &err {
                        _ => {
                            eprintln!("uncaught DbConnAcquireTimeoutSecsError");
                            std::process::exit(1); // We use exit for planned exits instead of panics
                        }
                    }
                })
            }
            None => DbConnAcquireTimeoutSecs::default(),
        };

        let conn_idle_timeout_secs = match &conf.conn_idle_timeout_secs {
            Some(raw_conn_idle_timeout_secs) => {
                DbConnIdleTimeoutSecs::new(raw_conn_idle_timeout_secs).unwrap_or_else(|err| {
                    match &err {
                        _ => {
                            eprintln!("uncaught DbConnIdleTimeoutSecsError");
                            std::process::exit(1); // We use exit for planned exits instead of panics
                        }
                    }
                })
            }
            None => DbConnIdleTimeoutSecs::default(),
        };

        let conn_max_lifetime_secs = match &conf.conn_max_lifetime_secs {
            Some(raw_conn_max_lifetime_secs) => {
                DbConnMaxLifetimeSecs::new(raw_conn_max_lifetime_secs).unwrap_or_else(|err| {
                    match &err {
                        _ => {
                            eprintln!("uncaught DbConnMaxLifetimeSecsError");
                            std::process::exit(1); // We use exit for planned exits instead of panics
                        }
                    }
                })
            }
            None => DbConnMaxLifetimeSecs::default(),
        };

        let max_connections = match &conf.max_connections {
            Some(raw_max_connections) => {
                DbMaxConnections::new(raw_max_connections).unwrap_or_else(|err| match &err {
                    _ => {
                        eprintln!("uncaught DbMaxConnectionsError");
                        std::process::exit(1); // We use exit for planned exits instead of panics
                    }
                })
            }
            None => DbMaxConnections::default(),
        };

        let min_connections = match &conf.min_connections {
            Some(raw_min_connections) => {
                DbMinConnections::new(raw_min_connections).unwrap_or_else(|err| match &err {
                    _ => {
                        eprintln!("uncaught DbMinConnectionsError");
                        std::process::exit(1); // We use exit for planned exits instead of panics
                    }
                })
            }
            None => DbMinConnections::default(),
        };

        Database {
            conn_string,
            conn_max_retries,
            conn_retry_init_delay_secs,

            conn_acquire_timeout_secs,
            conn_idle_timeout_secs,
            conn_max_lifetime_secs,

            max_connections,
            min_connections,
        }
    }

    pub fn validate_raw_database_config(
        &self,
        log_serv: &impl LoggingRepo,
        raw_db_conf: &ConfigDatabase,
    ) {
        // Validate raw database connection string file input
        match &raw_db_conf.conn_url_file {
            Some(raw_conn_url_file) => {
                if let Err(err) = DbConnectionString::new(raw_conn_url_file) {
                    match &err {
                        DbConnectionStringError::BadFileLoad(e) => {
                            log_serv.error(
                                module_path!(),
                                &format!("config database load connection file error: {}", e),
                            );
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config database connection string will be set to `{}`",
                                    DbConnectionString::default()
                                ),
                            );
                        }
                        DbConnectionStringError::EmptyConnectionString(_) => {
                            log_serv.warn(
                                module_path!(),
                                "config database connection string was empty",
                            );
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config database connection string will be set to `{}`",
                                    DbConnectionString::default()
                                ),
                            );
                        }
                        DbConnectionStringError::EmptyFilePath(_) => {
                            log_serv.warn(
                                module_path!(),
                                "config database connection file path was empty",
                            );
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config database connection string will be set to `{}`",
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
                        "config database connection string file was not specified, setting database connection string to `{}`",
                        DbConnectionString::default()
                    ),
                );
            }
        };

        // Validate raw database connection max retries input
        match &raw_db_conf.conn_max_retries {
            Some(raw_conn_max_retries) => {
                if let Err(err) = DbConnMaxRetries::new(raw_conn_max_retries) {
                    log_serv.error(
                        module_path!(),
                        &format!("config database connection max retries error: {}", err),
                    );
                    log_serv.warn(
                        module_path!(),
                        &format!(
                            "configdatabase connection max retries will be set to `{}`",
                            DbConnMaxRetries::default()
                        ),
                    );
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config database connection max retries was not specified, setting to default `{}`",
                        DbConnMaxRetries::default()
                    ),
                );
            }
        };

        // Validate raw database connection retry initial delay seconds input
        match &raw_db_conf.conn_retry_init_delay_secs {
            Some(raw_conn_retry_init_delay_secs) => {
                if let Err(err) = DbConnRetryInitDelaySecs::new(raw_conn_retry_init_delay_secs) {
                    log_serv.error(
                        module_path!(),
                        &format!(
                            "config database connection retry initial delay seconds error: {}",
                            err
                        ),
                    );
                    log_serv.warn(
                        module_path!(),
                        &format!(
                            "config database connection retry initial delay seconds will be set to `{}`",
                            DbConnRetryInitDelaySecs::default()
                        ),
                    );
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config database connection retry initial delay seconds was not specified, setting to default `{}`",
                        DbConnRetryInitDelaySecs::default()
                    ),
                );
            }
        }

        // Validate connection acquire timeout seconds input
        match &raw_db_conf.conn_acquire_timeout_secs {
            Some(raw_conn_acquire_timeout_secs) => {
                if let Err(err) = DbConnAcquireTimeoutSecs::new(raw_conn_acquire_timeout_secs) {
                    log_serv.error(
                        module_path!(),
                        &format!(
                            "config database connection acquire timeout seconds error: {}",
                            err
                        ),
                    );
                    log_serv.warn(
                        module_path!(),
                        &format!(
                            "config database connection acquire timeout seconds will be set to `{}`",
                            DbConnAcquireTimeoutSecs::default()
                        ),
                    );
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config database connection acquire timeout seconds was not specified, setting to default `{}`",
                        DbConnAcquireTimeoutSecs::default()
                    ),
                );
            }
        };

        // Validate connection idle timeout seconds input
        match &raw_db_conf.conn_idle_timeout_secs {
            Some(raw_conn_idle_timeout_secs) => {
                if let Err(err) = DbConnIdleTimeoutSecs::new(raw_conn_idle_timeout_secs) {
                    log_serv.error(
                        module_path!(),
                        &format!(
                            "config database connection idle timeout seconds error: {}",
                            err
                        ),
                    );
                    log_serv.warn(
                        module_path!(),
                        &format!(
                            "config database connection idle timeout seconds will be set to `{}`",
                            DbConnIdleTimeoutSecs::default()
                        ),
                    );
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config database connection idle timeout seconds was not specified, setting to default `{}`",
                        DbConnIdleTimeoutSecs::default()
                    ),
                );
            }
        };

        // Validate connection max lifetime seconds input
        match &raw_db_conf.conn_max_lifetime_secs {
            Some(raw_conn_max_lifetime_secs) => {
                if let Err(err) = DbConnMaxLifetimeSecs::new(raw_conn_max_lifetime_secs) {
                    log_serv.error(
                        module_path!(),
                        &format!(
                            "config database connection max lifetime seconds error: {}",
                            err
                        ),
                    );
                    log_serv.warn(
                        module_path!(),
                        &format!(
                            "config database connection max lifetime seconds will be set to `{}`",
                            DbConnMaxLifetimeSecs::default()
                        ),
                    );
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config database connection max lifetime seconds was not specified, setting to default `{}`",
                        DbConnMaxLifetimeSecs::default()
                    ),
                );
            }
        };

        // Validate min connections input
        match &raw_db_conf.min_connections {
            Some(raw_min_connections) => {
                if let Err(err) = DbMinConnections::new(raw_min_connections) {
                    log_serv.error(
                        module_path!(),
                        &format!("config database min connections error: {}", err),
                    );
                    log_serv.warn(
                        module_path!(),
                        &format!(
                            "config database min connections will be set to `{}`",
                            DbMinConnections::default()
                        ),
                    );
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config database min connections was not specified, setting to default `{}`",
                        DbMinConnections::default()
                    ),
                );
            }
        };

        // Validate max connections input
        match &raw_db_conf.max_connections {
            Some(raw_max_connections) => {
                if let Err(err) = DbMaxConnections::new(raw_max_connections) {
                    log_serv.error(
                        module_path!(),
                        &format!("config database max connections error: {}", err),
                    );
                    log_serv.warn(
                        module_path!(),
                        &format!(
                            "config database max connections will be set to `{}`",
                            DbMaxConnections::default()
                        ),
                    );
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config database max connections was not specified, setting to default `{}`",
                        DbMaxConnections::default()
                    ),
                );
            }
        };
    }
}
