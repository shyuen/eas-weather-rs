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

impl GetLatestAlertsError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseError(_) => "database_error",
            Self::DatabaseConnectionError(_) => "database_connection_error",
            Self::DataConversionError(_) => "data_conversion_error",
        }
    }
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

impl GetDailyAlertsError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseError(_) => "database_error",
            Self::DatabaseConnectionError(_) => "database_connection_error",
            Self::DataConversionError(_) => "data_conversion_error",
        }
    }
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

impl CreateAlertError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseError(_) => "database_error",
            Self::DatabaseConnectionError(_) => "database_connection_error",
            Self::ValidationError(_) => "validation_error",
        }
    }
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

impl UpdateAlertError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseError(_) => "database_error",
            Self::DatabaseConnectionError(_) => "database_connection_error",
            Self::ValidationError(_) => "validation_error",
            Self::NotFound => "not_found",
        }
    }
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

impl GetAlertError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseError(_) => "database_error",
            Self::DatabaseConnectionError(_) => "database_connection_error",
            Self::DataConversionError(_) => "data_conversion_error",
            Self::NotFound => "not_found",
        }
    }
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

impl PatchAlertError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseError(_) => "database_error",
            Self::DatabaseConnectionError(_) => "database_connection_error",
            Self::ValidationError(_) => "validation_error",
            Self::DataConversionError(_) => "data_conversion_error",
            Self::NotFound => "not_found",
        }
    }
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

impl DeleteAlertError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseError(_) => "database_error",
            Self::DatabaseConnectionError(_) => "database_connection_error",
            Self::NotFound => "not_found",
        }
    }
}

#[cfg(test)]
mod code_tests {
    use super::*;

    #[test]
    fn get_latest_alerts_error_codes() {
        assert_eq!(
            GetLatestAlertsError::DatabaseError("x".into()).code(),
            "database_error"
        );
        assert_eq!(
            GetLatestAlertsError::DatabaseConnectionError("x".into()).code(),
            "database_connection_error"
        );
        assert_eq!(
            GetLatestAlertsError::DataConversionError("x".into()).code(),
            "data_conversion_error"
        );
    }

    #[test]
    fn get_daily_alerts_error_codes() {
        assert_eq!(
            GetDailyAlertsError::DatabaseError("x".into()).code(),
            "database_error"
        );
        assert_eq!(
            GetDailyAlertsError::DatabaseConnectionError("x".into()).code(),
            "database_connection_error"
        );
        assert_eq!(
            GetDailyAlertsError::DataConversionError("x".into()).code(),
            "data_conversion_error"
        );
    }

    #[test]
    fn create_alert_error_codes() {
        assert_eq!(
            CreateAlertError::DatabaseError("x".into()).code(),
            "database_error"
        );
        assert_eq!(
            CreateAlertError::DatabaseConnectionError("x".into()).code(),
            "database_connection_error"
        );
        assert_eq!(
            CreateAlertError::ValidationError("x".into()).code(),
            "validation_error"
        );
    }

    #[test]
    fn update_alert_error_codes() {
        assert_eq!(
            UpdateAlertError::DatabaseError("x".into()).code(),
            "database_error"
        );
        assert_eq!(
            UpdateAlertError::DatabaseConnectionError("x".into()).code(),
            "database_connection_error"
        );
        assert_eq!(
            UpdateAlertError::ValidationError("x".into()).code(),
            "validation_error"
        );
        assert_eq!(UpdateAlertError::NotFound.code(), "not_found");
    }

    #[test]
    fn get_alert_error_codes() {
        assert_eq!(
            GetAlertError::DatabaseError("x".into()).code(),
            "database_error"
        );
        assert_eq!(
            GetAlertError::DatabaseConnectionError("x".into()).code(),
            "database_connection_error"
        );
        assert_eq!(
            GetAlertError::DataConversionError("x".into()).code(),
            "data_conversion_error"
        );
        assert_eq!(GetAlertError::NotFound.code(), "not_found");
    }

    #[test]
    fn patch_alert_error_codes() {
        assert_eq!(
            PatchAlertError::DatabaseError("x".into()).code(),
            "database_error"
        );
        assert_eq!(
            PatchAlertError::DatabaseConnectionError("x".into()).code(),
            "database_connection_error"
        );
        assert_eq!(
            PatchAlertError::ValidationError("x".into()).code(),
            "validation_error"
        );
        assert_eq!(
            PatchAlertError::DataConversionError("x".into()).code(),
            "data_conversion_error"
        );
        assert_eq!(PatchAlertError::NotFound.code(), "not_found");
    }

    #[test]
    fn delete_alert_error_codes() {
        assert_eq!(
            DeleteAlertError::DatabaseError("x".into()).code(),
            "database_error"
        );
        assert_eq!(
            DeleteAlertError::DatabaseConnectionError("x".into()).code(),
            "database_connection_error"
        );
        assert_eq!(DeleteAlertError::NotFound.code(), "not_found");
    }
}
