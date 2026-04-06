use crate::adaptors::axum::app_state::AppState;
use crate::domain::meta::port::MetaRepo;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;

pub async fn get_raw_app_config<MR>(State(state): State<AppState<MR>>) -> impl IntoResponse
where
    MR: MetaRepo,
{
    let app_data = state.get_meta_serv().get_app_data();

    Json(json!({
        "app_data": app_data,
    }))
}
