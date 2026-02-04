use crate::core::domain::config::webserver::Webserver;
use crate::core::ports::inbound::webserver::WebserverRepo;
use crate::core::ports::outbound::logging::LoggingRepo;

#[derive(Debug, Clone)]
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

    fn log_adaptor_config(&self, log_repo: &impl LoggingRepo, conf_webserv: &Webserver) {
        log_repo.info(
            module_path!(),
            &format!("webserver_hostname={}", conf_webserv.hostname.get()),
        );
        log_repo.info(
            module_path!(),
            &format!("webserver_port={}", conf_webserv.port.get()),
        );
        log_repo.info(
            module_path!(),
            &format!("webserver_base_path={}", conf_webserv.base_path.to_string()),
        );
        log_repo.info(
            module_path!(),
            &format!(
                "shutdown_timeout_secs={}",
                conf_webserv.shutdown_timeout_secs.get()
            ),
        );

        log_repo.info(module_path!(), &format!("api_key={}", conf_webserv.api_key));
        log_repo.info(module_path!(), &format!("jwt_key={}", conf_webserv.jwt_key));
        log_repo.info(
            module_path!(),
            &format!(
                "jwt_access_token_expiry_secs={}",
                conf_webserv.jwt_access_token_expiry_secs.get()
            ),
        );
    }

    // fn start_server(&self, config: &Webserver) {
    //     // Implementation to start the webserver using Poem framework
    // }
}
