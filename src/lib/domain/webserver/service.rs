use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
use crate::domain::webserver::model::{ShutdownReason, Webserver};
use crate::domain::webserver::port::WebserverPort;
use tracing::field::Empty;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct WebserverService<WP>
where
    WP: WebserverPort,
{
    pub repo: WP,
    conf: Webserver,
}

impl<WP> WebserverService<WP>
where
    WP: WebserverPort,
{
    /// Creates a new instance of WebserverService.
    pub fn new(conf_webserv: &Webserver) -> Self {
        let repo = WP::new(conf_webserv);
        Self {
            repo,
            conf: conf_webserv.clone(),
        }
    }

    /// Get the Webserver port
    pub fn get_webserver_port(&self) -> &WP {
        &self.repo
    }

    #[tracing::instrument(
        skip(self, alert_serv, config_serv, db_port),
        fields(operation = "start_server", result = Empty),
        level = "debug"
    )]
    pub async fn start_server<CP, AP, DP>(
        &self,
        alert_serv: &AlertService<AP>,
        config_serv: &ConfigService<CP>,
        db_port: &DP,
    ) -> Result<(), std::io::Error>
    where
        CP: ConfigPort,
        AP: AlertPort,
        DP: DatabasePort,
    {
        let span = tracing::Span::current();
        debug!(event_kind = "service", "start_server: starting server");
        info!(
            event_kind = "service",
            "start_server: starting {} server at {}:{}",
            self.repo.adaptor_name(),
            self.conf.hostname.get(),
            self.conf.port.get()
        );
        match self
            .repo
            .start_server(alert_serv, config_serv, db_port)
            .await
        {
            Ok(reason) => {
                span.record("result", "ok");
                match reason {
                    ShutdownReason::CtrlC => {
                        info!(
                            event_kind = "service",
                            "start_server: received Ctrl+C, shutting down"
                        )
                    }
                    ShutdownReason::Terminate => {
                        info!(
                            event_kind = "service",
                            "start_server: received terminate signal, shutting down"
                        )
                    }
                    ShutdownReason::Stopped => {
                        info!(event_kind = "service", "start_server: server stopped")
                    }
                }
                Ok(())
            }
            Err(err) => {
                error!(
                    event_kind = "service",
                    error_code = "start_server_io_error",
                    message = %err,
                    "start_server: server failed to start"
                );
                Err(err)
            }
        }
    }
}
