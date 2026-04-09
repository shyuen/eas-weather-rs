use crate::adaptors::axum::app_state::AppState;
use crate::domain::database::port::DatabaseRepo;
use crate::domain::meta::port::MetaRepo;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;

/// Handler for GET /meta/app_config
pub async fn get_raw_app_config<MR, DR>(State(state): State<AppState<MR, DR>>) -> impl IntoResponse
where
    MR: MetaRepo,
    DR: DatabaseRepo,
{
    let raw_conf = state.get_meta_repo().get_raw_config_data();

    Json(json!({
        "raw_conf": raw_conf,
    }))
}

pub async fn get_app_config<MR, DR>(State(state): State<AppState<MR, DR>>) -> impl IntoResponse
where
    MR: MetaRepo,
    DR: DatabaseRepo,
{
    let conf = state.get_meta_repo().get_conf();

    Json(json!({
        "conf": conf,
    }))
}
