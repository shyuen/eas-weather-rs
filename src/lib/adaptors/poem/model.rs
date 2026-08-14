use crate::adaptors::poem::handlers::meta::MetaHandler;
use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::adaptor_config::{AdaptorConfigField, AdaptorConfigRepr};
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;
use crate::domain::webserver::model::{ShutdownReason, Webserver};
use crate::domain::webserver::port::WebserverRepo;

use poem::{EndpointExt, Route, Server, listener::TcpListener};
use poem_openapi::{OpenApiService, Tags};
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct WebserverPoem {
    config: Webserver,
}

impl WebserverPoem {
    pub fn new(conf_webserv: Webserver) -> Self {
        WebserverPoem {
            config: conf_webserv,
        }
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
    fn new(conf_webserv: &Webserver) -> Self {
        WebserverPoem::new(conf_webserv.clone())
    }

    //async fn start_server<'a>(
    async fn start_server<D>(
        &self,
        config: &Webserver,
        alert_service: &AlertService<D>,
        meta_serv: &impl MetaPort,
    ) -> Result<ShutdownReason, std::io::Error>
    where
        D: DatabasePort + AlertPort,
    {
        // Construct root address
        let root_addr = format!("{}:{}", config.hostname.get(), config.port.get());

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
        let base_addr = format!("{}{}", root_addr, base_path);

        // Configure OpenAPI service
        let main_paths = OpenApiService::new(
            MetaHandler,
            env!("CARGO_PKG_NAME").to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
        .server(format!("http://{}", base_addr));

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
            .nest(format!("{}/docs", base_path), ui)
            .data(app_state.clone()); // Pass meta_service to the routes, requires EndpointExt

        // Database port for graceful shutdown is sourced from the alert service.
        let db_port = alert_service.get_db_port().clone();

        // Channel so the shutdown future can report why we stopped.
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<ShutdownReason>();

        // Start the server with graceful shutdown
        Server::new(TcpListener::bind(format!(
            "{}:{}",
            config.hostname.get(),
            config.port.get()
        )))
        .run_with_graceful_shutdown(
            routes,
            async move {
                let _ = tokio::signal::ctrl_c().await;
                let _ = shutdown_tx.send(ShutdownReason::CtrlC);

                // Perform any necessary cleanup here
                // e.g., close database connections, flush logs, etc.
                let _ = db_port.close_pool().await;
            },
            // Graceful shutdown timeout
            Some(Duration::from_secs(config.shutdown_timeout_secs.get())),
        )
        .await?;

        Ok(shutdown_rx.await.unwrap_or(ShutdownReason::Stopped))
    }
}

impl AdaptorConfigRepr for WebserverPoem {
    fn adaptor_name(&self) -> &'static str {
        "poem"
    }

    fn config_fields(&self) -> Vec<AdaptorConfigField> {
        let c = &self.config;
        vec![
            AdaptorConfigField::new("hostname", c.hostname.get().clone()),
            AdaptorConfigField::new("port", c.port.get().to_string()),
            AdaptorConfigField::new("base_path", c.base_path.to_string()),
            AdaptorConfigField::new(
                "shutdown_timeout_secs",
                c.shutdown_timeout_secs.get().to_string(),
            ),
            AdaptorConfigField::secret("api_key", c.api_key.to_string()),
            AdaptorConfigField::secret("jwt_key", c.jwt_key.to_string()),
            AdaptorConfigField::new(
                "jwt_access_token_expiry_secs",
                c.jwt_access_token_expiry_secs.get().to_string(),
            ),
        ]
    }
}
