use crate::adaptors::axum::api_errors::ApiError;
use axum::Json;
use axum::extract::Path;
use serde_json::{Value, json};

pub async fn list_error() -> Result<Json<Value>, ApiError> {
    Err(ApiError::InternalError)
}

pub async fn get_user(Path(id): Path<u32>) -> Result<Json<Value>, ApiError> {
    if id > 100 {
        return Err(ApiError::NotFound);
    }

    Ok(Json(json!({"id": id, "name": "User"})))
}
