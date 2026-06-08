use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;
use crate::domain::webserver::model::Webserver;

pub trait WebserverRepo: Clone + Send + Sync + 'static {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(conf_webserv: &Webserver) -> Self;

    /// Log configuration that was set for this service
    fn log_adaptor_config(&self, conf: &Webserver);

    /// Start the web server
    fn start_server<D>(
        &self,
        config: &Webserver,
        alert_service: &AlertService<D>,
        meta_port: &impl MetaPort,
    ) -> impl std::future::Future<Output = Result<(), std::io::Error>> + Send
    where
        D: DatabasePort + AlertPort;
}
