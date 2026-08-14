use crate::adaptors::axum::api_errors::ApiError;
use crate::adaptors::axum::handlers::error::ErrorResponse;
use axum::Json;
use axum::extract::Path;
use serde::Serialize;
use utoipa::ToSchema;

/// User payload returned by `/test/user/{id}`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: u32,
    pub name: String,
}

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
