use crate::adaptors::axum::api_errors::ApiError;
use axum::Json;
use axum::extract::Path;

use crate::adaptors::axum::openapi::{ErrorResponse, UserResponse};

/// Deliberately fails with a 500 error. Used to exercise the error path.
#[utoipa::path(
    get,
    path = "/test/error",
    responses(
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    tag = "test"
)]
pub async fn list_error() -> Result<Json<ErrorResponse>, ApiError> {
    Err(ApiError::InternalError)
}

/// Returns a fake user for the given id, or 404 if the id is too large.
#[utoipa::path(
    get,
    path = "/test/user/{id}",
    params(("id" = u32, Path, description = "User id")),
    responses(
        (status = 200, description = "A user", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    tag = "test"
)]
pub async fn get_user(Path(id): Path<u32>) -> Result<Json<UserResponse>, ApiError> {
    if id > 100 {
        return Err(ApiError::NotFound);
    }

    Ok(Json(UserResponse {
        id,
        name: "User".to_string(),
    }))
}
