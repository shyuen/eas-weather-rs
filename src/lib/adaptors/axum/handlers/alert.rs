//! HTTP handlers for the `/alerts` resource, one module per HTTP verb.
//!
//! Shared response/param contracts (`AlertSchema`, `AlertsListResponse`,
//! `LatestAlertsParams`) live here; each verb submodule owns its request DTO,
//! its `From` conversion into the domain input contract, its handler, and its
//! tests. The `pub(crate) use` re-exports at the bottom keep `routes.rs`,
//! `openapi.rs`, and `test_support.rs` importing from this module unchanged.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::domain::alert::model::Alert;

pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod patch;
pub(crate) mod post;
pub(crate) mod put;

pub(crate) use delete::delete_alert;
pub(crate) use get::{get_alerts, get_daily_alerts};
pub(crate) use patch::patch_alert;
pub(crate) use post::create_alert;
pub(crate) use put::update_alert;

#[derive(Deserialize, IntoParams)]
pub(crate) struct LatestAlertsParams {
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
}

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
    pub source: Option<String>,
    pub scope: String,
    pub references: Vec<String>,
}

impl From<&Alert> for AlertSchema {
    fn from(alert: &Alert) -> Self {
        AlertSchema {
            identifier: alert.identifier().as_str().to_string(),
            sender: alert.sender().as_str().to_string(),
            sent: alert.sent().as_offset_date_time().to_string(),
            status: alert.status().as_str().to_string(),
            msg_type: alert.msg_type().as_str().to_string(),
            source: alert.source().as_opt_str().map(|s| s.to_string()),
            scope: alert.scope().as_str().to_string(),
            references: alert
                .references()
                .as_db_string()
                .map(|s| s.split(' ').map(|x| x.to_string()).collect())
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
pub(crate) async fn body_to_json(response: axum::response::Response) -> serde_json::Value {
    use http_body_util::BodyExt;

    let (_, body) = response.into_parts();
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
