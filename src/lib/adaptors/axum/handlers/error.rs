use axum::Json;
use axum::extract::FromRequest;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::warn;
use utoipa::ToSchema;

/// Error body returned by endpoints that fail. Shared across all handlers.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// Extractor for a JSON request body that logs malformed bodies while still
/// returning axum's default `422 Unprocessable Entity` rejection.
pub struct JsonBody<T>(pub T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(JsonBody(value)),
            Err(rejection) => {
                warn!(
                    "rejected invalid JSON request body: {}",
                    rejection.body_text()
                );
                Err(rejection.into_response())
            }
        }
    }
}
