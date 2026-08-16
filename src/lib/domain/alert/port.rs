use thiserror::Error;

use crate::domain::alert::model::Alert;
use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;

pub trait AlertPort: Clone + Send + Sync + 'static {
    // Get alerts within 24 hours
    fn get_daily_alerts_data(
        &self,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<GetDailyAlertsResponse, GetDailyAlertsError>> + Send;

    // Get the latest alerts
    fn get_latest_alerts_data(
        &self,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<GetLatestAlertsResponse, GetLatestAlertsError>> + Send;

    // Create a new alert
    fn create_alert_data(
        &self,
        alert: Alert,
    ) -> impl Future<Output = Result<CreateAlertResponse, CreateAlertError>> + Send;

    // Update an existing alert (identified by `identifier`)
    fn update_alert_data(
        &self,
        identifier: &AlertIdentifier,
        alert: Alert,
    ) -> impl Future<Output = Result<UpdateAlertResponse, UpdateAlertError>> + Send;

    // Get a single alert by identifier
    fn get_alert_data(
        &self,
        identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<GetAlertResponse, GetAlertError>> + Send;

    // Delete an alert by identifier
    fn delete_alert_data(
        &self,
        identifier: &AlertIdentifier,
    ) -> impl Future<Output = Result<DeleteAlertResponse, DeleteAlertError>> + Send;
}

// Create Alert
pub struct CreateAlertResponse {
    pub alert: Alert,
}

// Update Alert
pub struct UpdateAlertResponse {
    pub alert: Alert,
}

// Get single alert by identifier
pub struct GetAlertResponse {
    pub alert: Alert,
}

// Patch (partially update) an alert
pub struct PatchAlertResponse {
    pub alert: Alert,
}

// Delete an alert
pub struct DeleteAlertResponse;

// Daily Alerts
pub struct GetDailyAlertsResponse {
    pub alerts: Vec<Alert>,
    pub total: u64,
}
// Latest Alerts
pub struct GetLatestAlertsResponse {
    pub alerts: Vec<Alert>,
    pub total: u64,
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

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CreateAlertError {
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("database error: {0}")]
    DatabaseConnectionError(String),
    #[error("validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UpdateAlertError {
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("database error: {0}")]
    DatabaseConnectionError(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("alert not found")]
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GetAlertError {
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("database error: {0}")]
    DatabaseConnectionError(String),
    #[error("conversion error: {0}")]
    DataConversionError(String),
    #[error("alert not found")]
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PatchAlertError {
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("database error: {0}")]
    DatabaseConnectionError(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("conversion error: {0}")]
    DataConversionError(String),
    #[error("alert not found")]
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DeleteAlertError {
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("database error: {0}")]
    DatabaseConnectionError(String),
    #[error("alert not found")]
    NotFound,
}
