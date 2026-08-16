use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::adaptor_config::AdaptorConfigRepr;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;
use crate::domain::webserver::model::{ShutdownReason, Webserver};

pub trait WebserverPort: AdaptorConfigRepr + Clone + Send + Sync + 'static {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(conf_webserv: &Webserver) -> Self;

    /// Start the web server. The returned `ShutdownReason` indicates why the
    /// server stopped so the caller (service) can log it; the adaptor performs
    /// the shutdown but does not own the logging.
    fn start_server<D>(
        &self,
        alert_service: &AlertService<D>,
        meta_port: &impl MetaPort,
    ) -> impl std::future::Future<Output = Result<ShutdownReason, std::io::Error>> + Send
    where
        D: DatabasePort + AlertPort;
}
