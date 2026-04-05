use crate::adaptors::axum::api_errors::ApiError;
use axum::Json;
use serde_json::Value;

pub async fn list_error() -> Result<Json<Value>, ApiError> {
    Err(ApiError::InternalError)
}
