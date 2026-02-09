use crate::core::domain::config::webserver::Webserver;
use crate::core::ports::inbound::webserver::WebserverRepo;
use crate::core::ports::outbound::database::DatabaseRepo;
use crate::core::ports::outbound::logging::LoggingRepo;

use poem::{Route, Server, listener::TcpListener};
use poem_openapi::OpenApiService;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct WebserverPoem {}

impl WebserverPoem {
    pub fn new() -> Self {
        WebserverPoem {}
    }
}

impl WebserverRepo for WebserverPoem {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(_log_repo: &impl LoggingRepo, _conf_webserv: &Webserver) -> Self {
        WebserverPoem::new()
    }

    fn log_adaptor_config(&self, log_repo: &impl LoggingRepo, conf_webserv: &Webserver) {
        log_repo.info(
            module_path!(),
            &format!("poem_hostname={}", conf_webserv.hostname.get()),
        );
        log_repo.info(
            module_path!(),
            &format!("poem_port={}", conf_webserv.port.get()),
        );
        log_repo.info(
            module_path!(),
            &format!("poem_base_path={}", conf_webserv.base_path.to_string()),
        );
        log_repo.info(
            module_path!(),
            &format!(
                "poem_shutdown_timeout_secs={}",
                conf_webserv.shutdown_timeout_secs.get()
            ),
        );

        log_repo.info(
            module_path!(),
            &format!("poem_api_key={}", conf_webserv.api_key),
        );
        log_repo.info(
            module_path!(),
            &format!("poem_jwt_key={}", conf_webserv.jwt_key),
        );
        log_repo.info(
            module_path!(),
            &format!(
                "poem_jwt_access_token_expiry_secs={}",
                conf_webserv.jwt_access_token_expiry_secs.get()
            ),
        );
    }

    async fn start_server(
        &self,
        config: &Webserver,
        log_repo: &impl LoggingRepo,
        db_repo: &impl DatabaseRepo,
    ) -> Result<(), std::io::Error> {
        // Construct root address
        let root_addr = format!("{}:{}", &config.hostname.get(), &config.port.get());

        // Construct base address for OpenAPI server
        let base_addr = format!("{}{:?}", &root_addr, &config.base_path.get());

        // Configure OpenAPI service
        let main_paths = OpenApiService::new(
            (),
            env!("CARGO_PKG_NAME").to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
        .server(format!("http://{}", &base_addr));

        // Create Swagger UI
        let ui = main_paths.swagger_ui();

        // Create routes
        let routes = Route::new().nest("/", main_paths).nest("/docs", ui);
        //.data(app_state.clone()); // Pass app_state to the routes

        // Start the server with graceful shutdown
        Server::new(TcpListener::bind(format!(
            "{}:{}",
            &config.hostname.get(),
            &config.port.get()
        )))
        .run_with_graceful_shutdown(
            routes,
            async move {
                let _ = tokio::signal::ctrl_c().await;

                log_repo.info(module_path!(), "shutdown signal received");
                log_repo.info(module_path!(), "commencing graceful shutdown");

                // Perform any necessary cleanup here
                // e.g., close database connections, flush logs, etc.
                let _ = db_repo.close_pool(*&log_repo).await;
            },
            // Graceful shutdown timeout
            Some(Duration::from_secs(*&config.shutdown_timeout_secs.get())),
        )
        .await
    }
}
