use crate::adaptors::axum::app_state::AppState;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;

/// Handler for GET /meta/raw_conf
pub(crate) async fn get_raw_app_config<MR, DR>(
    State(state): State<AppState<MR, DR>>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort,
{
    let raw_conf = state.get_meta_port().get_raw_config_data();

    Json(json!({
        "raw_conf": raw_conf,
    }))
}

/// Handler for GET /meta/conf
pub(crate) async fn get_app_config<MR, DR>(
    State(state): State<AppState<MR, DR>>,
) -> impl IntoResponse
where
    MR: MetaPort,
    DR: DatabasePort,
{
    let conf = state.get_meta_port().get_conf();

    Json(json!({
        "conf": conf,
    }))
}
