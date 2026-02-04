use crate::core::domain::config::webserver::ConfigWebserver;
use crate::core::ports::outbound::logging::LoggingRepo;
use crate::lib::application::ports::inbound::webserver::WebserverRepo;

pub struct WebserverPoem {}

impl WebserverPoem {
    pub fn new() -> Self {
        WebserverPoem {}
    }
}

impl WebserverRepo for WebserverPoem {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(log_repo: &impl LoggingRepo, conf_webserv: &Webserver) -> Self {
        WebserverPoem::new()
    }

    fn log_adaptor_config(&self, log_repo: &impl LoggingRepo, conf_webserv: &ConfigWebserver) {
        log_repo.info(
            module_path!(),
            &format!("webserver_hostname={}", conf_webserv.hostname.get()),
        );
    }

    fn start_server(&self, config: &ConfigWebserver) {
        // Implementation to start the webserver using Poem framework
    }
}
