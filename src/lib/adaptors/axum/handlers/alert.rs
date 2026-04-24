use crate::adaptors::axum::app_state::AppState;
use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaRepo;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;

/// Handler for GET /alerts/daily
pub(crate) async fn get_daily_alerts<MR, DR>(
    State(state): State<AppState<MR, DR>>,
) -> impl IntoResponse
where
    MR: MetaRepo,
    DR: DatabasePort,
{
    let data = state.get_db_repo();

    Json(json!({
        "conf": "moose",
    }))
}
