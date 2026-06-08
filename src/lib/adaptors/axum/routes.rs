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
