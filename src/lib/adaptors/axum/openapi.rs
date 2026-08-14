use serde::Serialize;
use utoipa::OpenApi;
use utoipa::ToSchema;

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
#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct AlertsListResponse {
    total: u64,
    count: u64,
    limit: u64,
    offset: u64,
    alerts: Vec<AlertSchema>,
}

/// Single alert, mirroring the serialized shape of `domain::alert::model::Alert`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct AlertSchema {
    identifier: String,
    sender: String,
    sent: String,
    status: String,
    msg_type: String,
    source: String,
    scope: String,
    references: Vec<String>,
}

/// Error body returned by endpoints that fail.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct ErrorResponse {
    error: String,
}

/// Raw configuration dump returned by `/meta/raw_conf`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct RawConfResponse {
    raw_conf: serde_json::Value,
}

/// Processed configuration returned by `/meta/conf`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct ConfResponse {
    conf: serde_json::Value,
}

/// User payload returned by `/test/user/{id}`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct UserResponse {
    id: u32,
    name: String,
}
