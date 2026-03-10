use crate::core::domain::config::webserver::Webserver;
use crate::core::domain::meta::ports::Meta;
use crate::core::ports::inbound::webserver::WebserverRepo;
use crate::core::ports::outbound::database::DatabaseRepo;
use crate::core::ports::outbound::logging::LoggingRepo;

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tokio::signal;

#[derive(Debug, Clone)]
pub struct WebserverAxum {}

impl WebserverAxum {
    pub fn new() -> Self {
        WebserverAxum {}
    }

    async fn shutdown_signal(log_repo: impl LoggingRepo, db_repo: impl DatabaseRepo) {
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
                log_repo.info(module_path!(), &format!("ctrl_c received"));
            },
            _ = terminate => {
                log_repo.info(module_path!(), &format!("terminate signal received"));
            },
        }

        // Perform other tasks as necessary
        let _ = db_repo.close_pool(&log_repo).await; // Close DB connection pool

        log_repo.info(module_path!(), &format!("goodbye"));
    }
}

impl WebserverRepo for WebserverAxum {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(_log_repo: &impl LoggingRepo, _conf_webserv: &Webserver) -> Self {
        WebserverAxum::new()
    }

    fn log_adaptor_config(&self, log_repo: &impl LoggingRepo, conf_webserv: &Webserver) {
        log_repo.info(
            module_path!(),
            &format!("axum_hostname={}", conf_webserv.hostname.get()),
        );
        log_repo.info(
            module_path!(),
            &format!("axum_port={}", conf_webserv.port.get()),
        );
        log_repo.info(
            module_path!(),
            &format!("axum_base_path={}", conf_webserv.base_path.to_string()),
        );
        log_repo.info(
            module_path!(),
            &format!(
                "axum_shutdown_timeout_secs={}",
                conf_webserv.shutdown_timeout_secs.get()
            ),
        );

        log_repo.info(
            module_path!(),
            &format!("axum_api_key={}", conf_webserv.api_key),
        );
        log_repo.info(
            module_path!(),
            &format!("axum_jwt_key={}", conf_webserv.jwt_key),
        );
    }

    async fn start_server(
        &self,
        config: &Webserver,
        log_repo: &impl LoggingRepo,
        db_repo: &impl DatabaseRepo,
        meta: &impl Meta,
    ) -> Result<(), std::io::Error> {
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .with_state(db_repo.clone())
            .with_state(log_repo.clone())
            .with_state(meta.clone());

        let addr = format!("{}:{}", config.hostname.get(), config.port.get());
        log_repo.info(module_path!(), &format!("starting axum server at {}", addr));

        // run our app with hyper, listening globally on port 3000
        let listener: TcpListener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        axum::serve(listener, app)
            .with_graceful_shutdown(WebserverAxum::shutdown_signal(
                log_repo.clone(),
                db_repo.clone(),
            ))
            .await
    }
}
