use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
use crate::domain::logging::adaptor_config::AdaptorConfigRepr;
use crate::domain::webserver::model::{ShutdownReason, Webserver};

pub trait WebserverPort: AdaptorConfigRepr + Clone + Send + Sync + 'static {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(conf_webserv: &Webserver) -> Self;

    /// Start the web server. The returned `ShutdownReason` indicates why the
    /// server stopped so the caller (service) can log it; the adaptor performs
    /// the shutdown but does not own the logging.
    fn start_server<CP, AP, DP>(
        &self,
        alert_service: &AlertService<AP>,
        config_service: &ConfigService<CP>,
        db_port: &DP,
    ) -> impl std::future::Future<Output = Result<ShutdownReason, std::io::Error>> + Send
    where
        CP: ConfigPort,
        AP: AlertPort,
        DP: DatabasePort;
}
