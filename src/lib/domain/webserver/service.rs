use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::service::MetaService;
use crate::domain::webserver::model::ShutdownReason;
use crate::domain::webserver::port::WebserverPort;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct WebserverService<WP>
where
    WP: WebserverPort,
{
    pub repo: WP,
}

impl<WP> WebserverService<WP>
where
    WP: WebserverPort,
{
    /// Creates a new instance of WebserverService.
    pub fn new<C>(conf_serv: &ConfigService<C>) -> Self
    where
        C: ConfigPort,
    {
        let conf_webserv = conf_serv.get_webservicer_config();

        let repo = WP::new(conf_webserv);
        Self { repo }
    }

    /// Get the Webserver repository
    pub fn get_port(&self) -> &WP {
        &self.repo
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

        debug!("start_server: starting server");
        info!(
            "start_server: starting {} server at {}:{}",
            self.repo.adaptor_name(),
            webserv_conf.hostname.get(),
            webserv_conf.port.get()
        );
        match self
            .repo
            .start_server(webserv_conf, alert_serv, meta_serv)
            .await
        {
            Ok(reason) => {
                match reason {
                    ShutdownReason::CtrlC => info!("start_server: received Ctrl+C, shutting down"),
                    ShutdownReason::Terminate => {
                        info!("start_server: received terminate signal, shutting down")
                    }
                    ShutdownReason::Stopped => {
                        info!("start_server: server stopped")
                    }
                }
                Ok(())
            }
            Err(err) => {
                error!("start_server: server failed to start: {}", err);
                Err(err)
            }
        }
    }
}
