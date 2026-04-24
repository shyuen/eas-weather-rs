use crate::domain::database::port::DatabasePort;
use crate::domain::logging::port::LoggingPort;
use crate::domain::meta::port::MetaPort;
use crate::domain::webserver::model::Webserver;

pub trait WebserverRepo: Clone + Send + Sync + 'static {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(log_port: &impl LoggingPort, conf_webserv: &Webserver) -> Self;

    /// Log configuration that was set for this service
    fn log_adaptor_config(&self, log_port: &impl LoggingPort, conf: &Webserver);

    /// Start the web server
    fn start_server(
        &self,
        config: &Webserver,
        log_port: &impl LoggingPort,
        db_port: &impl DatabasePort,
        meta_port: &impl MetaPort,
    ) -> impl std::future::Future<Output = Result<(), std::io::Error>> + Send;
}
