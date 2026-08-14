use utoipa::OpenApi;

/// The OpenAPI document for this service, collected from the annotated handlers.
///
/// Response schemas live with their handlers (`handlers/alert.rs`, `handlers/meta.rs`,
/// `handlers/test.rs`) or centrally for shared contracts (`handlers/error.rs`).
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
        crate::adaptors::axum::handlers::alert::AlertsListResponse,
        crate::adaptors::axum::handlers::alert::AlertSchema,
        crate::adaptors::axum::handlers::error::ErrorResponse,
        crate::adaptors::axum::handlers::meta::RawConfResponse,
        crate::adaptors::axum::handlers::meta::ConfResponse,
        crate::adaptors::axum::handlers::test::UserResponse
    ))
)]
pub(crate) struct ApiDoc;
