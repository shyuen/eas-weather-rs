use serde::Serialize;
use utoipa::OpenApi;
use utoipa::ToSchema;

use crate::domain::alert::model::Alert;
use crate::domain::config::model::Config;
use crate::domain::meta::port::ValidatedConfig;

/// The OpenAPI document for this service, collected from the annotated handlers.
#[derive(Debug, OpenApi)]
#[openapi(
    info(
        title = "EAS Weather",
        description = "Emergency alert system (EAS) web microservice",
        version = "0.1.0"
    ),
    paths(
        crate::adaptors::axum::handlers::alert::get_alerts,
        crate::adaptors::axum::handlers::alert::get_daily_alerts,
        crate::adaptors::axum::handlers::meta::get_app_config,
        crate::adaptors::axum::handlers::meta::get_raw_app_config,
        crate::adaptors::axum::handlers::test::list_error,
        crate::adaptors::axum::handlers::test::get_user
    ),
    components(schemas(
        AlertsListResponse,
        AlertSchema,
        ErrorResponse,
        RawConfResponse,
        ConfResponse,
        UserResponse
    ))
)]
pub(crate) struct ApiDoc;

/// Paginated list of alerts returned by the alert endpoints.
///
/// `alerts` carries the domain `Alert` type directly (so the response is the
/// real serialized data), while the schema for that field is documented via
/// [`AlertSchema`] through utoipa's `value_type`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AlertsListResponse {
    pub total: u64,
    pub count: u64,
    pub limit: u64,
    pub offset: u64,
    #[schema(value_type = Vec<AlertSchema>)]
    pub alerts: Vec<Alert>,
}

/// Single alert, mirroring the serialized shape of `domain::alert::model::Alert`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AlertSchema {
    pub identifier: String,
    pub sender: String,
    pub sent: String,
    pub status: String,
    pub msg_type: String,
    pub source: String,
    pub scope: String,
    pub references: Vec<String>,
}

/// Error body returned by endpoints that fail.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// Raw configuration dump returned by `/meta/raw_conf`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RawConfResponse {
    #[schema(value_type = Object)]
    pub raw_conf: Config,
}

/// Processed configuration returned by `/meta/conf`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConfResponse {
    #[schema(value_type = Object)]
    pub conf: ValidatedConfig,
}

/// User payload returned by `/test/user/{id}`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: u32,
    pub name: String,
}
