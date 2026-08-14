use serde::Serialize;
use utoipa::ToSchema;

/// Error body returned by endpoints that fail. Shared across all handlers.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}
