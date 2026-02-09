use crate::core::ports::inbound::config::ConfigRepo;
use crate::core::ports::inbound::webserver::WebserverRepo;
use crate::core::ports::outbound::database::DatabaseRepo;
use crate::core::ports::outbound::logging::LoggingRepo;
use crate::core::services::config::ConfigService;
use crate::core::services::database::DatabaseService;
use crate::core::services::logging::LoggingService;

#[derive(Debug, Clone)]
pub struct WebserverService<W>
where
    W: WebserverRepo,
{
    pub repo: W,
}

impl<W> WebserverService<W>
where
    W: WebserverRepo,
{
    /// Creates a new instance of WebserverService.
    pub fn new<C, L>(conf_serv: &ConfigService<C>, log_serv: &LoggingService<L>) -> Self
    where
        C: ConfigRepo,
        L: LoggingRepo,
    {
        let log_repo = log_serv.get_repo();
        let conf_webserv = conf_serv.get_webservicer_config();

        let repo = W::new(log_repo, conf_webserv);
        Self { repo }
    }

    /// Get the Webserver repository
    pub fn get_repo(&self) -> &W {
        &self.repo
    }

    /// Log configuration that's currently set
    pub fn log_adaptor_config<L, C>(
        &self,
        log_serv: &LoggingService<L>,
        conf_serv: &ConfigService<C>,
    ) where
        C: ConfigRepo,
        L: LoggingRepo,
    {
        let log_repo = log_serv.get_repo();
        let conf_webserver = conf_serv.get_webservicer_config();
        self.repo.log_adaptor_config(log_repo, conf_webserver);
    }

    pub async fn start_server<C, D, L>(
        &self,
        conf_serv: &ConfigService<C>,
        db_serv: &DatabaseService<D>,
        log_serv: &LoggingService<L>,
    ) -> Result<(), std::io::Error>
    where
        L: LoggingRepo,
        D: DatabaseRepo,
        C: ConfigRepo,
    {
        let webserv_conf = conf_serv.get_webservicer_config();
        let db_repo = db_serv.get_repo();
        let log_repo = log_serv.get_repo();

        self.repo
            .start_server(webserv_conf, log_repo, db_repo)
            .await
    }
}
