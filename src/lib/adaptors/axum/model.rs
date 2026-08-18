use crate::adaptors::axum::app_state::AppState;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::oneshot;

use crate::adaptors::axum::routes::create_routes;
use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::adaptor_config::{AdaptorConfigField, AdaptorConfigRepr};
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
use crate::domain::webserver::model::{ShutdownReason, Webserver};
use crate::domain::webserver::port::WebserverPort;

#[derive(Debug, Clone)]
pub struct WebserverAxum {
    config: Webserver,
}

impl WebserverAxum {
    pub fn new(conf_webserv: Webserver) -> Self {
        WebserverAxum {
            config: conf_webserv,
        }
    }
}

impl WebserverPort for WebserverAxum {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(conf_webserv: &Webserver) -> Self {
        WebserverAxum::new(conf_webserv.clone())
    }

    async fn start_server<C, AP, DP>(
        &self,
        alert_service: &AlertService<AP>,
        config_service: &ConfigService<C>,
        db_port: &DP,
    ) -> Result<ShutdownReason, std::io::Error>
    where
        C: ConfigPort,
        AP: AlertPort,
        DP: DatabasePort,
    {
        // Database port for graceful shutdown
        let db_port = db_port.clone();

        // Create the application state with the necessary services
        let state = AppState::new(config_service.clone(), alert_service.clone());

        // Create the Axum application with the defined routes and state
        let app = create_routes().with_state(state);

        let addr = format!("{}:{}", self.config.hostname.get(), self.config.port.get());

        // Channel so shutdown_signal can report why we stopped.
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<ShutdownReason>();

        // Start listening to the TCP port
        let listener: TcpListener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        // Start the Axum server
        axum::serve(listener, app)
            .with_graceful_shutdown(WebserverAxum::shutdown_signal(db_port, shutdown_tx))
            .await?;

        Ok(shutdown_rx.await.unwrap_or(ShutdownReason::Stopped))
    }
}

impl WebserverAxum {
    async fn shutdown_signal(
        db_port: impl DatabasePort,
        shutdown_tx: oneshot::Sender<ShutdownReason>,
    ) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                let _ = shutdown_tx.send(ShutdownReason::CtrlC);
            },
            _ = terminate => {
                let _ = shutdown_tx.send(ShutdownReason::Terminate);
            },
        }

        // Perform other tasks as necessary
        let _ = db_port.close_pool().await; // Close DB connection pool
    }
}

/// Implementation of the AdaptorConfigRepr trait for WebserverAxum
impl AdaptorConfigRepr for WebserverAxum {
    fn adaptor_name(&self) -> &'static str {
        "axum"
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
        ]
    }
}

/// The body of an [Meta] creation request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GetMetaHttpRequestBody {
    name: String,
    email_address: String,
}

/// The response body data field for successful [Author] creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetMetaResponseData {
    id: String,
}
