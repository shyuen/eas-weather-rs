use crate::domain::config::model::Config;
use crate::domain::config::port::ConfigRepo;
use crate::domain::config::service::ConfigService;
use crate::domain::meta::port::MetaRepo;

use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::webserver::model::Webserver;

#[derive(Debug, Clone)]
pub struct MetaService<C>
where
    C: ConfigRepo,
{
    //conf_repo: &'a C,
    conf_repo: C,
}

//impl<'a, C> Service<'a, C>
impl<C> MetaService<C>
where
    C: ConfigRepo,
{
    //pub fn new(conf_serv: &'a ConfigService<C>) -> Self
    pub fn new(conf_serv: ConfigService<C>) -> Self
    where
        C: ConfigRepo,
    {
        let conf_repo = conf_serv.get_repo().clone();
        Self { conf_repo }
    }

    /// Get the Config repository
    pub fn get_repo(&self) -> &C {
        &self.conf_repo
    }
}

impl<C> MetaRepo for MetaService<C>
where
    C: ConfigRepo,
{
    fn get_raw_config_data(&self) -> Config {
        self.conf_repo.get_raw_config().clone()
    }

    // Return a validated configuration struct
    // which can be used by a handler
    fn get_conf(&self) -> ValidatedConfig {
        ValidatedConfig::new(
            self.conf_repo.get_logging_config().clone(),
            self.conf_repo.get_database_config().clone(),
            self.conf_repo.get_webserver_config().clone(),
        )
    }
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
