use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NotFound,             // 404
    InvalidInput(String), // 400
    InternalError,        // 500
}

/// Conversion from `ApiError` to an HTTP response, mapping each error variant to an appropriate status code and error message in the response body.
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        // Map each error variant to an appropriate status code and error message.
        let (status, error_message) = match self {
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource Not Found".to_string()),
            ApiError::InvalidInput(msg) => {
                (StatusCode::BAD_REQUEST, format!("Invalid Input: {}", msg))
            }
        };

        // Create a JSON response body containing the error message.
        let body = Json(json!({
            "error": error_message
        }));

        // Return the HTTP response with the status code and JSON body.
        (status, body).into_response()
    }
}
