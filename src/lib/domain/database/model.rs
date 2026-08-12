use serde::{Deserialize, Serialize};

use crate::domain::config::model::ConfigDatabase;
use crate::domain::database::new_types::db_conn_acquire_timeout_secs::DbConnAcquireTimeoutSecs;
use crate::domain::database::new_types::db_conn_idle_timeout_secs::DbConnIdleTimeoutSecs;
use crate::domain::database::new_types::db_conn_init_delay_secs::DbConnRetryInitDelaySecs;
use crate::domain::database::new_types::db_conn_max_lifetime_secs::DbConnMaxLifetimeSecs;
use crate::domain::database::new_types::db_conn_max_retries::DbConnMaxRetries;
use crate::domain::database::new_types::db_conn_string::{
    DbConnectionString, DbConnectionStringError,
};
use crate::domain::database::new_types::db_max_connections::DbMaxConnections;
use crate::domain::database::new_types::db_min_connections::DbMinConnections;
use crate::domain::utils::helpers::serialize_with_display;
use crate::warn_config_invalid;
use crate::warn_config_load_failed;
use crate::warn_config_not_specified;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    //#[serde(skip_serializing)]
    #[serde(serialize_with = "serialize_with_display")]
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

    pub fn validate_raw_config(&self, raw_db_conf: &ConfigDatabase) {
        // Validate raw database connection string file input
        match &raw_db_conf.conn_url_file {
            Some(raw_conn_url_file) => {
                if let Err(err) = DbConnectionString::new(raw_conn_url_file) {
                    match &err {
                        DbConnectionStringError::BadFileLoad(e) => {
                            warn_config_load_failed!(
                                "database.conn_string",
                                raw_conn_url_file,
                                &e.to_string(),
                                DbConnectionString::default(),
                            );
                        }
                        DbConnectionStringError::EmptyConnectionString(_) => {
                            warn_config_invalid!(
                                "database.conn_string",
                                "empty",
                                DbConnectionString::default(),
                            );
                        }
                        DbConnectionStringError::EmptyFilePath(_) => {
                            warn_config_invalid!(
                                "database.conn_string_file",
                                raw_conn_url_file,
                                DbConnectionString::default(),
                            );
                        }
                    }
                }
            }
            None => {
                warn_config_not_specified!(
                    "database.conn_string_file",
                    DbConnectionString::default(),
                );
            }
        };

        // Validate raw database connection max retries input
        match &raw_db_conf.conn_max_retries {
            Some(raw_conn_max_retries) => {
                if DbConnMaxRetries::new(raw_conn_max_retries).is_err() {
                    warn_config_invalid!(
                        "database.conn_max_retries",
                        raw_conn_max_retries,
                        DbConnMaxRetries::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!(
                    "database.conn_max_retries",
                    DbConnMaxRetries::default()
                );
            }
        };

        // Validate raw database connection retry initial delay seconds input
        match &raw_db_conf.conn_retry_init_delay_secs {
            Some(raw_conn_retry_init_delay_secs) => {
                if DbConnRetryInitDelaySecs::new(raw_conn_retry_init_delay_secs).is_err() {
                    warn_config_invalid!(
                        "database.conn_retry_init_delay_secs",
                        raw_conn_retry_init_delay_secs,
                        DbConnRetryInitDelaySecs::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!(
                    "database.conn_retry_init_delay_secs",
                    DbConnRetryInitDelaySecs::default(),
                );
            }
        }

        // Validate connection acquire timeout seconds input
        match &raw_db_conf.conn_acquire_timeout_secs {
            Some(raw_conn_acquire_timeout_secs) => {
                if DbConnAcquireTimeoutSecs::new(raw_conn_acquire_timeout_secs).is_err() {
                    warn_config_invalid!(
                        "database.conn_acquire_timeout_secs",
                        raw_conn_acquire_timeout_secs,
                        DbConnAcquireTimeoutSecs::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!(
                    "database.conn_acquire_timeout_secs",
                    DbConnAcquireTimeoutSecs::default(),
                );
            }
        };

        // Validate connection idle timeout seconds input
        match &raw_db_conf.conn_idle_timeout_secs {
            Some(raw_conn_idle_timeout_secs) => {
                if DbConnIdleTimeoutSecs::new(raw_conn_idle_timeout_secs).is_err() {
                    warn_config_invalid!(
                        "database.conn_idle_timeout_secs",
                        raw_conn_idle_timeout_secs,
                        DbConnIdleTimeoutSecs::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!(
                    "database.conn_idle_timeout_secs",
                    DbConnIdleTimeoutSecs::default(),
                );
            }
        };

        // Validate connection max lifetime seconds input
        match &raw_db_conf.conn_max_lifetime_secs {
            Some(raw_conn_max_lifetime_secs) => {
                if DbConnMaxLifetimeSecs::new(raw_conn_max_lifetime_secs).is_err() {
                    warn_config_invalid!(
                        "database.conn_max_lifetime_secs",
                        raw_conn_max_lifetime_secs,
                        DbConnMaxLifetimeSecs::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!(
                    "database.conn_max_lifetime_secs",
                    DbConnMaxLifetimeSecs::default(),
                );
            }
        };

        // Validate min connections input
        match &raw_db_conf.min_connections {
            Some(raw_min_connections) => {
                if DbMinConnections::new(raw_min_connections).is_err() {
                    warn_config_invalid!(
                        "database.min_connections",
                        raw_min_connections,
                        DbMinConnections::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!("database.min_connections", DbMinConnections::default());
            }
        };

        // Validate max connections input
        match &raw_db_conf.max_connections {
            Some(raw_max_connections) => {
                if DbMaxConnections::new(raw_max_connections).is_err() {
                    warn_config_invalid!(
                        "database.max_connections",
                        raw_max_connections,
                        DbMaxConnections::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!("database.max_connections", DbMaxConnections::default());
            }
        };
    }
}
