use crate::core::ports::inbound::config::ConfigRepo;
use crate::core::ports::inbound::webserver::WebserverRepo;
use crate::core::ports::outbound::logging::LoggingRepo;
use crate::core::services::config::ConfigService;
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
}
