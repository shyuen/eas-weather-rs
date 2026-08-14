use axum::Router;
use axum::routing::get;

use crate::adaptors::axum::app_state::AppState;
use crate::adaptors::axum::handlers::alert::{get_alerts, get_daily_alerts};
use crate::adaptors::axum::handlers::health::{liveness, readiness, startup};
use crate::adaptors::axum::handlers::meta::{get_app_config, get_raw_app_config};
use crate::adaptors::axum::handlers::test::{get_user, list_error};
use crate::domain::alert::port::AlertPort;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;

pub fn create_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/health", create_health_routes())
        .nest("/test", create_test_routes())
        .nest("/meta", create_meta_routes())
        .nest("/alerts", create_alert_routes())
}

pub fn create_health_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/startup", get(startup))
        .route("/readiness", get(readiness))
        .route("/liveness", get(liveness))
}

pub fn create_test_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/error", get(list_error))
        .route("/user/{id}", get(get_user))
}

pub fn create_meta_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/raw_conf", get(get_raw_app_config))
        .route("/conf", get(get_app_config))
}

pub fn create_alert_routes<MR, DR>() -> Router<AppState<MR, DR>>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    Router::new()
        .route("/", get(get_alerts))
        .route("/daily", get(get_daily_alerts))
}

#[cfg(test)]
mod tests {
    use crate::test_support::{
        DEFAULT_PAGE_LIMIT, MockDb, MockMeta, PAGE_LIMIT_MAX, build_full_app, build_state,
        build_webserver,
    };
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn get_status(uri: &str) -> u16 {
        let state = build_state::<MockDb>(MockMeta::new(build_webserver(
            DEFAULT_PAGE_LIMIT,
            PAGE_LIMIT_MAX,
        )));
        let app = build_full_app::<MockDb>(state);
        let response = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        response.status().as_u16()
    }

    #[tokio::test]
    async fn root_returns_hello_world() {
        assert_eq!(get_status("/").await, 200);
    }

    #[tokio::test]
    async fn health_routes_are_mounted() {
        assert_eq!(get_status("/health/startup").await, 200);
        assert_eq!(get_status("/health/readiness").await, 200);
        assert_eq!(get_status("/health/liveness").await, 200);
    }

    #[tokio::test]
    async fn meta_routes_are_mounted() {
        assert_eq!(get_status("/meta/conf").await, 200);
    }

    #[tokio::test]
    async fn alert_routes_are_mounted() {
        assert_eq!(get_status("/alerts").await, 200);
        assert_eq!(get_status("/alerts/daily").await, 200);
    }

    #[tokio::test]
    async fn test_routes_are_mounted() {
        assert_eq!(get_status("/test/user/1").await, 200);
    }
}
