use axum::Json;
use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::warn;
use utoipa::ToSchema;

use crate::domain::alert::port::{
    CreateAlertError, GetDailyAlertsError, GetLatestAlertsError, UpdateAlertError,
};

/// Machine-readable error code returned in every error response body.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// A domain field failed validation.
    AlertValidationFailed,
    /// The requested alert does not exist.
    AlertNotFound,
    /// The database returned an error for the query.
    DatabaseError,
    /// The connection to the database could not be established.
    DatabaseConnectionError,
    /// The request body could not be deserialised.
    InvalidRequestBody,
    /// The identifier in the URL path is not a valid alert identifier.
    InvalidIdentifier,
    /// An unexpected internal error occurred.
    InternalError,
}

/// Unified error body returned by all endpoints.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    pub status: u16,
}

impl ApiErrorResponse {
    pub fn new(code: ErrorCode, message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            code,
            message: message.into(),
            status: status.as_u16(),
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

// ── From domain errors ────────────────────────────────────────────────────────

impl From<CreateAlertError> for ApiErrorResponse {
    fn from(err: CreateAlertError) -> Self {
        match err {
            CreateAlertError::ValidationError(msg) => ApiErrorResponse::new(
                ErrorCode::AlertValidationFailed,
                msg,
                StatusCode::BAD_REQUEST,
            ),
            CreateAlertError::DatabaseError(msg) => ApiErrorResponse::new(
                ErrorCode::DatabaseError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CreateAlertError::DatabaseConnectionError(msg) => ApiErrorResponse::new(
                ErrorCode::DatabaseConnectionError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

impl From<UpdateAlertError> for ApiErrorResponse {
    fn from(err: UpdateAlertError) -> Self {
        match err {
            UpdateAlertError::ValidationError(msg) => ApiErrorResponse::new(
                ErrorCode::AlertValidationFailed,
                msg,
                StatusCode::BAD_REQUEST,
            ),
            UpdateAlertError::NotFound => ApiErrorResponse::new(
                ErrorCode::AlertNotFound,
                "alert not found",
                StatusCode::NOT_FOUND,
            ),
            UpdateAlertError::DatabaseError(msg) => ApiErrorResponse::new(
                ErrorCode::DatabaseError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            UpdateAlertError::DatabaseConnectionError(msg) => ApiErrorResponse::new(
                ErrorCode::DatabaseConnectionError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

impl From<GetLatestAlertsError> for ApiErrorResponse {
    fn from(err: GetLatestAlertsError) -> Self {
        match err {
            GetLatestAlertsError::DatabaseError(msg) => ApiErrorResponse::new(
                ErrorCode::DatabaseError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            GetLatestAlertsError::DatabaseConnectionError(msg) => ApiErrorResponse::new(
                ErrorCode::DatabaseConnectionError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            GetLatestAlertsError::DataConversionError(msg) => ApiErrorResponse::new(
                ErrorCode::InternalError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

impl From<GetDailyAlertsError> for ApiErrorResponse {
    fn from(err: GetDailyAlertsError) -> Self {
        match err {
            GetDailyAlertsError::DatabaseError(msg) => ApiErrorResponse::new(
                ErrorCode::DatabaseError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            GetDailyAlertsError::DatabaseConnectionError(msg) => ApiErrorResponse::new(
                ErrorCode::DatabaseConnectionError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            GetDailyAlertsError::DataConversionError(msg) => ApiErrorResponse::new(
                ErrorCode::InternalError,
                msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

/// Extractor for a JSON request body that logs malformed bodies while still
/// returning axum's default `422 Unprocessable Entity` rejection — now wrapped
/// in the unified [`ApiErrorResponse`] envelope.
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
                let message = rejection.body_text();
                warn!("rejected invalid JSON request body: {}", message);
                Err(ApiErrorResponse::new(
                    ErrorCode::InvalidRequestBody,
                    message,
                    StatusCode::UNPROCESSABLE_ENTITY,
                )
                .into_response())
            }
        }
    }
}
