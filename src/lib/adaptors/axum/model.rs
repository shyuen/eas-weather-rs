use crate::domain::database::port::DatabaseRepo;
use crate::domain::logging::port::LoggingRepo;
use crate::domain::meta::port::MetaRepo;
use crate::domain::webserver::model::Webserver;
use crate::domain::webserver::port::WebserverRepo;

use axum::extract::{Path, State};
use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;

#[derive(Debug, Clone)]
pub struct WebserverAxum {}

#[derive(Debug)]
enum ApiError {
    NotFound,             // 404
    InvalidInput(String), // 400
    InternalError,        // 500
}

// Axum turns return values into HTTP responses with the IntoReponse trait
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "data not found".to_string()),
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal Server error".to_string(),
            ),
        };

        let body = Json(json! ({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "ok", "message": "Server is running"}))
}

async fn list_users() -> Result<Json<Value>, ApiError> {
    Err(ApiError::InternalError)
}

async fn get_user(Path(id): Path<u32>) -> Result<Json<Value>, ApiError> {
    if id > 100 {
        return Err(ApiError::NotFound);
    }

    Ok(Json(json!({"id": id, "name": "User"})))
}

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
        meta_serv: &impl MetaRepo,
    ) -> Result<(), std::io::Error> {
        let state = AppState {
            meta_serv: Arc::new(meta_serv.clone()),
        };

        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/health", get(health_check))
            .route("/users", get(list_users))
            .route("/user/{:id}", get(get_user))
            .nest("/meta", api_routes())
            .with_state(state);

        let addr = format!("{}:{}", config.hostname.get(), config.port.get());
        log_repo.info(module_path!(), &format!("starting axum server at {}", addr));

        // Start listening to the TCP port
        let listener: TcpListener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        // Start the Axum server
        axum::serve(listener, app)
            .with_graceful_shutdown(WebserverAxum::shutdown_signal(
                log_repo.clone(),
                db_repo.clone(),
            ))
            .await
    }
}

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
struct AppState<M: MetaRepo> {
    meta_serv: Arc<M>,
}

fn api_routes<M: MetaRepo>() -> Router<AppState<M>> {
    //Router::new().route("/authors", get(meta_info::<M>))
    Router::new().route("/authors", get(list_users))
}

async fn meta_info<M: MetaRepo>(
    State(state): State<AppState<M>>,
    //Json(body): Json<CreateMetaHttpRequestBody>,
) -> Result<ApiSuccess<GetMetaResponseData>, ApiError> {
    //Result<ApiSuccess<GetMetaHttpRequestBody>, ApiError>
    let data = state.meta_serv.get_app_data().await;
    //Ok(Json(json!(data)))
    //
    // Ok(ApiSuccess(StatusCode::OK, Json(ApiResponseBody {
    //     status_code: 200,
    //     data: GetMetaHttpRequestBody {
    //         name: "test".to_string(),
    //         email_address: "".to_string(),
    //     },
    // )))
    //
    Ok(ApiSuccess(
        StatusCode::OK,
        Json(ApiResponseBody {
            status_code: 200,
            data: GetMetaResponseData {
                id: "Test".to_string(),
            },
        }),
    ))
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

#[derive(Debug, Clone)]
pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<ApiResponseBody<T>>);

/// Generic response structure shared by all API responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    //use serde_json::Value;
    use tower::util::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        //let app = create_app();
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/users", get(list_users))
            .route("/user/{:id}", get(get_user));

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        // Oneshot comes from the tower trait
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        //let body = response.collect().await.unwrap();

        // let json: Value = serde_json::from_str(body).unwrap();

        // assert_eq!(json["status"], "ok");
        // assert_eq!(json["message"], "server is running");
    }

    #[tokio::test]
    async fn test_api_error_into_response() {
        let test_cases = vec![
            (ApiError::NotFound, StatusCode::NOT_FOUND),
            (
                ApiError::InvalidInput("bad data".to_string()),
                StatusCode::BAD_REQUEST,
            ),
            (ApiError::InternalError, StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for (error, expected_status) in test_cases {
            let response = error.into_response();

            assert_eq!(response.status(), expected_status);
        }
    }
}
