use crate::adaptors::axum::app_state::AppState;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

use crate::adaptors::axum::routes::create_routes;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;
use crate::domain::webserver::model::Webserver;
use crate::domain::webserver::port::WebserverRepo;

#[derive(Debug, Clone)]
pub struct WebserverAxum {}

impl WebserverAxum {
    pub fn new() -> Self {
        WebserverAxum {}
    }

    async fn shutdown_signal(db_port: impl DatabasePort) {
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
                info!("ctrl_c received");
            },
            _ = terminate => {
                info!("terminate signal received");
            },
        }

        // Perform other tasks as necessary
        let _ = db_port.close_pool().await; // Close DB connection pool

        info!("goodbye from axum");
    }
}

impl WebserverRepo for WebserverAxum {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(_conf_webserv: &Webserver) -> Self {
        WebserverAxum::new()
    }

    fn log_adaptor_config(&self, conf_webserv: &Webserver) {
        info!("axum_hostname={}", conf_webserv.hostname.to_string());
        info!("axum_port={}", conf_webserv.port.get());
        info!("axum_base_path={}", conf_webserv.base_path.to_string());
        info!(
            "axum_shutdown_timeout_secs={}",
            conf_webserv.shutdown_timeout_secs.get()
        );

        info!("axum_api_key={}", conf_webserv.api_key.to_string());
        info!("axum_jwt_key={}", conf_webserv.jwt_key.to_string());
    }

    async fn start_server(
        &self,
        config: &Webserver,
        db_port: &impl DatabasePort,
        meta_port: &impl MetaPort,
    ) -> Result<(), std::io::Error> {
        // Create the application state with the necessary services
        let state = AppState::new(meta_port.clone(), db_port.clone());

        // Create the Axum application with the defined routes and state
        let app = create_routes().with_state(state);

        let addr = format!("{}:{}", config.hostname.get(), config.port.get());
        info!("starting axum server at {}", addr);

        // Start listening to the TCP port
        let listener: TcpListener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        // Start the Axum server
        axum::serve(listener, app)
            .with_graceful_shutdown(WebserverAxum::shutdown_signal(db_port.clone()))
            .await
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

// #[cfg(test)]
// mod tests {
//     use axum::http::StatusCode;
//     use axum::{body::Body, http::Request};
//     use tower::util::ServiceExt;

//     use axum::{Router, response::IntoResponse, routing::get};

//     use super::*;

//     #[tokio::test]
//     async fn test_health_check() {
//         //let app = create_app();
//         let app = Router::new()
//             //.route("/health", get(health_check))
//             .route("/users", get(list_users))
//             .route("/user/{:id}", get(get_user));

//         let request = Request::builder()
//             .uri("/health")
//             .body(Body::empty())
//             .unwrap();

//         // Oneshot comes from the tower trait
//         let response = app.oneshot(request).await.unwrap();

//         assert_eq!(response.status(), StatusCode::OK);

//         //let body = response.collect().await.unwrap();

//         // let json: Value = serde_json::from_str(body).unwrap();

//         // assert_eq!(json["status"], "ok");
//         // assert_eq!(json["message"], "server is running");
//     }

//     #[tokio::test]
//     async fn test_api_error_into_response() {
//         let test_cases = vec![
//             (ApiError::NotFound, StatusCode::NOT_FOUND),
//             (
//                 ApiError::InvalidInput("bad data".to_string()),
//                 StatusCode::BAD_REQUEST,
//             ),
//             (ApiError::InternalError, StatusCode::INTERNAL_SERVER_ERROR),
//         ];

//         for (error, expected_status) in test_cases {
//             let response = error.into_response();

//             assert_eq!(response.status(), expected_status);
//         }
//     }
// }
