use crate::domain::config::model::Config;
use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::webserver::model::Webserver;

//use std::future::Future;

pub trait MetaRepo: Clone + Send + Sync + 'static {
    /// Get application metadata, such as version info, uptime, etc.
    fn get_raw_config_data(&self) -> Config;

    /// Get the validated configuration struct which can be used by handlers
    fn get_conf(&self) -> ValidatedConfig;
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidatedConfig {
    conf_logging: Logging,
    conf_database: Database,
    conf_webserver: Webserver,
}

impl ValidatedConfig {
    pub fn new(conf_logging: Logging, conf_database: Database, conf_webserver: Webserver) -> Self {
        Self {
            conf_logging,
            conf_database,
            conf_webserver,
        }
    }

    pub fn get_logging_config(&self) -> &Logging {
        &self.conf_logging
    }

    pub fn get_database_config(&self) -> &Database {
        &self.conf_database
    }

    pub fn get_webserver_config(&self) -> &Webserver {
        &self.conf_webserver
    }
}
