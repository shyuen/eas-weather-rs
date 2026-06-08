use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
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
    pub fn new<C>(conf_serv: &ConfigService<C>) -> Self
    where
        C: ConfigPort,
    {
        let conf_webserv = conf_serv.get_webservicer_config();

        let repo = WR::new(conf_webserv);
        Self { repo }
    }

    /// Get the Webserver repository
    pub fn get_port(&self) -> &WR {
        &self.repo
    }

    /// Log configuration that's currently set
    pub fn log_adaptor_config<C>(&self, conf_serv: &ConfigService<C>)
    where
        C: ConfigPort,
    {
        let conf_webserver = conf_serv.get_webservicer_config();
        self.repo.log_adaptor_config(conf_webserver);
    }

    pub async fn start_server<C, D>(
        &self,
        conf_serv: &ConfigService<C>,
        alert_serv: &AlertService<D>,
        meta_serv: &MetaService<C>,
    ) -> Result<(), std::io::Error>
    where
        D: DatabasePort + AlertPort,
        C: ConfigPort,
    {
        let webserv_conf = conf_serv.get_webservicer_config();

        self.repo
            .start_server(webserv_conf, alert_serv, meta_serv)
            .await
    }
}
