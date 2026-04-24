use thiserror::Error;

use crate::domain::alert::model::Alert;

pub trait DatabasePortAlert: Clone + Send + Sync + 'static {
    // Get alerts within 24 hours
    fn get_daily_alerts_data(
        &self,
    ) -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send;
}

// Daily Alerts
pub struct GetDailyAlertsResponse {
    pub alerts: Vec<Alert>,
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
