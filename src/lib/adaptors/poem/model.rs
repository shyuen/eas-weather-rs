use crate::adaptors::poem::handlers::meta::MetaHandler;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;
use crate::domain::webserver::model::Webserver;
use crate::domain::webserver::port::WebserverRepo;
use tracing::info;

use poem::{EndpointExt, Route, Server, listener::TcpListener};
use poem_openapi::{OpenApiService, Tags};
use std::sync::Arc;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct WebserverPoem {}

impl WebserverPoem {
    pub fn new() -> Self {
        WebserverPoem {}
    }
}

#[derive(Debug, Clone)]
//The global application state shared between all request handlers.
pub struct AppState<M: MetaPort> {
    pub meta: Arc<M>,
    pub moose: String,
}

#[derive(Tags)]
pub enum OperationalTags {
    /// Metadata endpoints
    Meta,
}

impl WebserverRepo for WebserverPoem {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(_conf_webserv: &Webserver) -> Self {
        WebserverPoem::new()
    }

    fn log_adaptor_config(&self, conf_webserv: &Webserver) {
        info!("poem_hostname={}", conf_webserv.hostname.get());
        info!("poem_port={}", conf_webserv.port.get());
        info!("poem_base_path={}", conf_webserv.base_path.to_string());
        info!(
            "poem_shutdown_timeout_secs={}",
            conf_webserv.shutdown_timeout_secs.get()
        );

        info!("poem_api_key={}", conf_webserv.api_key);
        info!("poem_jwt_key={}", conf_webserv.jwt_key);
        info!(
            "poem_jwt_access_token_expiry_secs={}",
            conf_webserv.jwt_access_token_expiry_secs.get()
        );
    }

    //async fn start_server<'a>(
    async fn start_server(
        &self,
        config: &Webserver,
        db_port: &impl DatabasePort,
        meta_serv: &impl MetaPort,
    ) -> Result<(), std::io::Error> {
        // Construct root address
        let root_addr = format!("{}:{}", &config.hostname.get(), &config.port.get());

        // Check the base_path value
        let base_path: &str = match &config.base_path.get() {
            Some(base_path) => {
                let mut clean_base_path = base_path.to_string();

                // Ensure it begins with `/`
                if !base_path.starts_with("/") {
                    clean_base_path = "/".to_string() + &clean_base_path;
                };

                &clean_base_path.to_string()
            }
            None => "/",
        };

        // Construct base address for OpenAPI server
        let base_addr = format!("{}{}", &root_addr, &base_path);

        info!("poem server path: http://{}", &base_addr);

        // Configure OpenAPI service
        let main_paths = OpenApiService::new(
            MetaHandler,
            env!("CARGO_PKG_NAME").to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
        .server(format!("http://{}", &base_addr));

        // Create Swagger UI
        let ui = main_paths.swagger_ui();

        // Construct dependencies to inject into handlers.
        let app_state = AppState {
            meta: Arc::new(meta_serv.clone()),
            moose: "string".to_string(),
        };

        // Create routes
        let routes = Route::new()
            .nest("/", main_paths)
            .nest(format!("{}/docs", &base_path), ui)
            .data(app_state.clone()); // Pass meta_service to the routes, requires EndpointExt

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

                info!("shutdown signal received");
                info!("commencing graceful shutdown");

                // Perform any necessary cleanup here
                // e.g., close database connections, flush logs, etc.
                let _ = db_port.close_pool().await;
            },
            // Graceful shutdown timeout
            Some(Duration::from_secs(*&config.shutdown_timeout_secs.get())),
        )
        .await
    }
}
