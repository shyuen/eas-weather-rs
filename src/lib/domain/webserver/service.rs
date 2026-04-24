use crate::domain::alert::port::DatabasePortAlert;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
use crate::domain::database::service::DatabaseService;
use crate::domain::logging::port::LoggingPort;
use crate::domain::logging::service::LoggingService;
use crate::domain::meta::service::MetaService;
use crate::domain::webserver::port::WebserverRepo;

#[derive(Debug, Clone)]
pub struct WebserverService<WR>
where
    WR: WebserverRepo,
{
    pub repo: WR,
}

impl<WR> WebserverService<WR>
where
    WR: WebserverRepo,
{
    /// Creates a new instance of WebserverService.
    pub fn new<C, L>(conf_serv: &ConfigService<C>, log_serv: &LoggingService<L>) -> Self
    where
        C: ConfigPort,
        L: LoggingPort,
    {
        let log_port = log_serv.get_port();
        let conf_webserv = conf_serv.get_webservicer_config();

        let repo = WR::new(log_port, conf_webserv);
        Self { repo }
    }

    /// Get the Webserver repository
    pub fn get_port(&self) -> &WR {
        &self.repo
    }

    /// Log configuration that's currently set
    pub fn log_adaptor_config<L, C>(
        &self,
        log_serv: &LoggingService<L>,
        conf_serv: &ConfigService<C>,
    ) where
        C: ConfigPort,
        L: LoggingPort,
    {
        let log_port = log_serv.get_port();
        let conf_webserver = conf_serv.get_webservicer_config();
        self.repo.log_adaptor_config(log_port, conf_webserver);
    }

    pub async fn start_server<C, D, L>(
        &self,
        conf_serv: &ConfigService<C>,
        db_serv: &DatabaseService<D>,
        log_serv: &LoggingService<L>,
        meta_serv: &MetaService<C>,
    ) -> Result<(), std::io::Error>
    where
        L: LoggingPort,
        D: DatabasePort + DatabasePortAlert,
        C: ConfigPort,
    {
        let webserv_conf = conf_serv.get_webservicer_config();
        let db_port = db_serv.get_port();
        let log_port = log_serv.get_port();

        self.repo
            .start_server(webserv_conf, log_port, db_port, meta_serv)
            .await
    }
}
