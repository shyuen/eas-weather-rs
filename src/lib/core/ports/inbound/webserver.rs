use crate::core::domain::config::webserver::Webserver;
use crate::core::ports::outbound::database::DatabaseRepo;
use crate::core::ports::outbound::logging::LoggingRepo;

pub trait WebserverRepo: Clone + Send + Sync + 'static {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(log_repo: &impl LoggingRepo, conf_webserv: &Webserver) -> Self;

    /// Log configuration that was set for this service
    fn log_adaptor_config(&self, log_repo: &impl LoggingRepo, conf: &Webserver);

    /// Start the web server
    fn start_server(
        &self,
        config: &Webserver,
        log_repo: &impl LoggingRepo,
        db_repo: &impl DatabaseRepo,
    ) -> impl std::future::Future<Output = Result<(), std::io::Error>> + Send;
}
