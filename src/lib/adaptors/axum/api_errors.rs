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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;

    async fn into_response_json(err: ApiError) -> (StatusCode, serde_json::Value) {
        let response = err.into_response();
        let status = response.status();
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn internal_error_maps_to_500() {
        let (status, json) = into_response_json(ApiError::InternalError).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["error"], "Internal Server Error");
    }

    #[tokio::test]
    async fn not_found_maps_to_404() {
        let (status, json) = into_response_json(ApiError::NotFound).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["error"], "Resource Not Found");
    }

    #[tokio::test]
    async fn invalid_input_maps_to_400_with_message() {
        let (status, json) = into_response_json(ApiError::InvalidInput("bad limit".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["error"], "Invalid Input: bad limit");
    }
}
