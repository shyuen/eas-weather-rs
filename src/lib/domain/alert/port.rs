use thiserror::Error;

use crate::domain::alert::model::Alert;

pub trait AlertPort: Clone + Send + Sync + 'static {
    // Get alerts within 24 hours
    fn get_daily_alerts_data(
        &self,
    ) -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send;

    // Get the latest 100 alerts
    fn get_latest_alerts_data(
        &self,
    ) -> impl Future<Output = Result<GetLatestAlertsResponse, GetLatestAlertsError>> + Send;
}

// Daily Alerts
pub struct GetDailyAlertsResponse {
    pub alerts: Vec<Alert>,
}
// Latest Alerts
pub struct GetLatestAlertsResponse {
    pub alerts: Vec<Alert>,
}
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GetLatestAlertsError {
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("database error: {0}")]
    DatabaseConnectionError(String),
    #[error("conversion error: {0}")]
    DataConversionError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GetDailyAlertsError {
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("database error: {0}")]
    DatabaseConnectionError(String),
    #[error("conversion error: {0}")]
    DataConversionError(String),
}
