use crate::domain::alert::model::{Alert, CreateAlertInput, PatchAlertInput, UpdateAlertInput};
use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::new_types::alert_msg_type::AlertMsgType;
use crate::domain::alert::new_types::alert_references::AlertReferences;
use crate::domain::alert::new_types::alert_references::ExtendedMessageIdentifier;
use crate::domain::alert::new_types::alert_scope::AlertScope;
use crate::domain::alert::new_types::alert_sender::AlertSender;
use crate::domain::alert::new_types::alert_sent::AlertSent;
use crate::domain::alert::new_types::alert_source::AlertSource;
use crate::domain::alert::new_types::alert_status::AlertStatus;
use crate::domain::alert::port::{
    AlertPort, CreateAlertError, CreateAlertResponse, DeleteAlertError, DeleteAlertResponse,
    GetAlertError, GetDailyAlertsError, GetDailyAlertsResponse, GetLatestAlertsError,
    GetLatestAlertsResponse, PatchAlertError, PatchAlertResponse, UpdateAlertError,
    UpdateAlertResponse,
};
use crate::domain::database::port::DatabasePort;
use crate::domain::database::service::DatabaseService;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::{debug, error, info, warn};

/// The AlertService struct provides methods for managing alerts.
/// Contains ports to interact with other services
#[derive(Debug, Clone)]
pub struct AlertService<D>
where
    D: AlertPort + DatabasePort,
{
    db_port: D,
}

impl<D> AlertService<D>
where
    D: AlertPort + DatabasePort,
{
    /// Creates a new instance of AlertService.
    pub fn new(data_serv: &DatabaseService<D>) -> Self {
        let db_port = data_serv.get_database_port();

        // Log the initialization of the AlertService
        info!("Initializing AlertService");

        Self {
            db_port: db_port.clone(),
        }
    }

    /// Get the Database port
    pub fn get_database_port(&self) -> &D {
        &self.db_port
    }

    /// Retrieve the latest alerts sent within the last 24 hours from the database.
    pub async fn get_daily_alerts(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<GetDailyAlertsResponse, GetDailyAlertsError> {
        debug!("get_daily_alerts(limit={}, offset={})", limit, offset);
        match self.db_port.get_daily_alerts_data(limit, offset).await {
            Ok(resp) => {
                info!(
                    "get_daily_alerts returned {} of {} alerts",
                    resp.alerts.len(),
                    resp.total
                );
                Ok(resp)
            }
            Err(err) => {
                error!("get_daily_alerts failed: {}", err);
                Err(err)
            }
        }
    }

    /// Retrieve the latest alerts from the database.
    pub async fn get_latest_alerts(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<GetLatestAlertsResponse, GetLatestAlertsError> {
        debug!("get_latest_alerts(limit={}, offset={})", limit, offset);
        match self.db_port.get_latest_alerts_data(limit, offset).await {
            Ok(resp) => {
                info!(
                    "get_latest_alerts returned {} of {} alerts",
                    resp.alerts.len(),
                    resp.total
                );
                Ok(resp)
            }
            Err(err) => {
                error!("get_latest_alerts failed: {}", err);
                Err(err)
            }
        }
    }

    /// Validate and persist a new alert to the database.
    pub async fn create_alert(
        &self,
        input: CreateAlertInput,
    ) -> Result<CreateAlertResponse, CreateAlertError> {
        let alert = match build_alert(input) {
            Ok(alert) => alert,
            Err(err) => {
                warn!("create_alert rejected invalid input: {}", err);
                return Err(CreateAlertError::ValidationError(err));
            }
        };

        debug!("create_alert(alert={:?})", alert);
        match self.db_port.create_alert_data(alert).await {
            Ok(resp) => {
                info!("create_alert persisted successfully");
                Ok(resp)
            }
            Err(err) => {
                error!("create_alert failed: {}", err);
                Err(err)
            }
        }
    }

    /// Validate and replace an existing alert (identified by `identifier`).
    pub async fn update_alert(
        &self,
        identifier: AlertIdentifier,
        input: UpdateAlertInput,
    ) -> Result<UpdateAlertResponse, UpdateAlertError> {
        let alert = match build_alert_for_update(identifier, input) {
            Ok(alert) => alert,
            Err(err) => {
                warn!("update_alert rejected invalid input: {}", err);
                return Err(UpdateAlertError::ValidationError(err));
            }
        };

        debug!("update_alert(alert={:?})", alert);
        let alert_identifier = alert.identifier().clone();
        match self
            .db_port
            .update_alert_data(&alert_identifier, alert)
            .await
        {
            Ok(resp) => {
                info!("update_alert persisted successfully");
                Ok(resp)
            }
            Err(err) => {
                error!("update_alert failed: {}", err);
                Err(err)
            }
        }
    }

    /// Validate and partially update an existing alert (identified by
    /// `identifier`). Fields present in `input` override the stored value;
    /// fields absent are left untouched.
    pub async fn patch_alert(
        &self,
        identifier: AlertIdentifier,
        input: PatchAlertInput,
    ) -> Result<PatchAlertResponse, PatchAlertError> {
        let existing = match self.db_port.get_alert_data(&identifier).await {
            Ok(resp) => resp.alert,
            Err(err) => {
                error!("patch_alert failed to fetch existing alert: {}", err);
                return Err(match err {
                    GetAlertError::NotFound => PatchAlertError::NotFound,
                    GetAlertError::DatabaseError(msg) => PatchAlertError::DatabaseError(msg),
                    GetAlertError::DatabaseConnectionError(msg) => {
                        PatchAlertError::DatabaseConnectionError(msg)
                    }
                    GetAlertError::DataConversionError(msg) => {
                        PatchAlertError::DataConversionError(msg)
                    }
                });
            }
        };

        let alert = match build_alert_for_patch(identifier, &existing, input) {
            Ok(alert) => alert,
            Err(err) => {
                warn!("patch_alert rejected invalid input: {}", err);
                return Err(PatchAlertError::ValidationError(err));
            }
        };

        debug!("patch_alert(alert={:?})", alert);
        let alert_identifier = alert.identifier().clone();
        match self
            .db_port
            .update_alert_data(&alert_identifier, alert)
            .await
        {
            Ok(resp) => {
                info!("patch_alert persisted successfully");
                Ok(PatchAlertResponse { alert: resp.alert })
            }
            Err(err) => {
                error!("patch_alert failed: {}", err);
                Err(match err {
                    UpdateAlertError::NotFound => PatchAlertError::NotFound,
                    UpdateAlertError::DatabaseError(msg) => PatchAlertError::DatabaseError(msg),
                    UpdateAlertError::DatabaseConnectionError(msg) => {
                        PatchAlertError::DatabaseConnectionError(msg)
                    }
                    UpdateAlertError::ValidationError(msg) => PatchAlertError::ValidationError(msg),
                })
            }
        }
    }

    /// Delete an existing alert (identified by `identifier`).
    pub async fn delete_alert(
        &self,
        identifier: AlertIdentifier,
    ) -> Result<DeleteAlertResponse, DeleteAlertError> {
        debug!("delete_alert(identifier={})", identifier.as_str());
        match self.db_port.delete_alert_data(&identifier).await {
            Ok(resp) => {
                info!("delete_alert removed successfully");
                Ok(resp)
            }
            Err(err) => {
                error!("delete_alert failed: {}", err);
                Err(err)
            }
        }
    }
}

/// Build a validated [`Alert`] from raw input, returning the first validation
/// error message on failure.
fn build_alert(input: CreateAlertInput) -> Result<Alert, String> {
    let identifier = AlertIdentifier::new(input.identifier).map_err(|e| e.to_string())?;
    let sender = AlertSender::new(input.sender).map_err(|e| e.to_string())?;
    let sent_ts = OffsetDateTime::parse(&input.sent, &Rfc3339)
        .map_err(|e| format!("invalid `sent` timestamp: {}", e))?;
    let sent = AlertSent::new(sent_ts).map_err(|e| e.to_string())?;
    let status = AlertStatus::new(input.status).map_err(|e| e.to_string())?;
    let msg_type = AlertMsgType::new(input.msg_type).map_err(|e| e.to_string())?;
    let source = AlertSource::new(&input.source.unwrap_or_default()).map_err(|e| e.to_string())?;
    let scope = AlertScope::new(input.scope).map_err(|e| e.to_string())?;

    let mut refs = Vec::new();
    for r in input.references {
        refs.push(ExtendedMessageIdentifier::new(&r).map_err(|e| e.to_string())?);
    }
    let references = AlertReferences::new(refs).map_err(|e| e.to_string())?;

    Ok(Alert::new(
        identifier, sender, sent, status, msg_type, source, scope, references,
    ))
}

/// Build a validated [`Alert`] for an update, applying the path-supplied
/// `identifier` authoritatively (the body carries no identifier).
fn build_alert_for_update(
    identifier: AlertIdentifier,
    input: UpdateAlertInput,
) -> Result<Alert, String> {
    let sender = AlertSender::new(input.sender).map_err(|e| e.to_string())?;
    let sent_ts = OffsetDateTime::parse(&input.sent, &Rfc3339)
        .map_err(|e| format!("invalid `sent` timestamp: {}", e))?;
    let sent = AlertSent::new(sent_ts).map_err(|e| e.to_string())?;
    let status = AlertStatus::new(input.status).map_err(|e| e.to_string())?;
    let msg_type = AlertMsgType::new(input.msg_type).map_err(|e| e.to_string())?;
    let source = AlertSource::new(&input.source.unwrap_or_default()).map_err(|e| e.to_string())?;
    let scope = AlertScope::new(input.scope).map_err(|e| e.to_string())?;

    let mut refs = Vec::new();
    for r in input.references {
        refs.push(ExtendedMessageIdentifier::new(&r).map_err(|e| e.to_string())?);
    }
    let references = AlertReferences::new(refs).map_err(|e| e.to_string())?;

    Ok(Alert::new(
        identifier, sender, sent, status, msg_type, source, scope, references,
    ))
}

/// Build a validated [`Alert`] for a partial update, merging the provided patch
/// fields over an existing alert. Fields absent from the patch keep their
/// existing validated values; an explicit `null` on a required field is
/// rejected, while optional fields (`source`, `references`) are cleared.
fn build_alert_for_patch(
    identifier: AlertIdentifier,
    existing: &Alert,
    input: PatchAlertInput,
) -> Result<Alert, String> {
    let sender = match input.sender {
        Some(Some(sender)) => AlertSender::new(sender).map_err(|e| e.to_string())?,
        Some(None) => return Err("field `sender` cannot be null".to_string()),
        None => existing.sender().clone(),
    };
    let sent = match input.sent {
        Some(Some(sent)) => {
            let sent_ts = OffsetDateTime::parse(&sent, &Rfc3339)
                .map_err(|e| format!("invalid `sent` timestamp: {}", e))?;
            AlertSent::new(sent_ts).map_err(|e| e.to_string())?
        }
        Some(None) => return Err("field `sent` cannot be null".to_string()),
        None => existing.sent().clone(),
    };
    let status = match input.status {
        Some(Some(status)) => AlertStatus::new(status).map_err(|e| e.to_string())?,
        Some(None) => return Err("field `status` cannot be null".to_string()),
        None => existing.status().clone(),
    };
    let msg_type = match input.msg_type {
        Some(Some(msg_type)) => AlertMsgType::new(msg_type).map_err(|e| e.to_string())?,
        Some(None) => return Err("field `msg_type` cannot be null".to_string()),
        None => existing.msg_type().clone(),
    };
    let source = match input.source {
        Some(Some(source)) => AlertSource::new(&source).map_err(|e| e.to_string())?,
        Some(None) => AlertSource::new("").map_err(|e| e.to_string())?,
        None => existing.source().clone(),
    };
    let scope = match input.scope {
        Some(Some(scope)) => AlertScope::new(scope).map_err(|e| e.to_string())?,
        Some(None) => return Err("field `scope` cannot be null".to_string()),
        None => existing.scope().clone(),
    };
    let references = match input.references {
        Some(Some(references)) => {
            let mut refs = Vec::new();
            for r in references {
                refs.push(ExtendedMessageIdentifier::new(&r).map_err(|e| e.to_string())?);
            }
            AlertReferences::new(refs).map_err(|e| e.to_string())?
        }
        Some(None) => AlertReferences::new(Vec::new()).map_err(|e| e.to_string())?,
        None => existing.references().clone(),
    };

    Ok(Alert::new(
        identifier, sender, sent, status, msg_type, source, scope, references,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::database::service::DatabaseService;
    use crate::test_support::{FailingDb, MockConfig, MockDb};

    /// Build an `AlertService<D>` over the supplied `DatabasePort + AlertPort` double.
    fn build_service<D>() -> AlertService<D>
    where
        D: AlertPort + DatabasePort,
    {
        let conf_service = crate::domain::config::service::ConfigService {
            port: MockConfig::new(),
        };
        let db_service = DatabaseService::<D>::new(&conf_service);
        AlertService::new(&db_service)
    }

    #[tokio::test]
    async fn get_latest_alerts_returns_port_data() {
        let service = build_service::<MockDb>();
        let resp = service.get_latest_alerts(10, 5).await.unwrap();
        assert_eq!(resp.total, 42);
        assert!(resp.alerts.is_empty());
    }

    #[tokio::test]
    async fn get_latest_alerts_propagates_error() {
        let service = build_service::<FailingDb>();
        let result = service.get_latest_alerts(10, 5).await;
        assert!(matches!(
            result,
            Err(GetLatestAlertsError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn get_daily_alerts_returns_port_data() {
        let service = build_service::<MockDb>();
        let resp = service.get_daily_alerts(10, 5).await.unwrap();
        assert_eq!(resp.total, 42);
        assert!(resp.alerts.is_empty());
    }

    #[tokio::test]
    async fn get_daily_alerts_propagates_error() {
        let service = build_service::<FailingDb>();
        let result = service.get_daily_alerts(10, 5).await;
        assert!(matches!(
            result,
            Err(GetDailyAlertsError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn create_alert_returns_port_data() {
        let service = build_service::<MockDb>();
        let input = build_test_input();
        let resp = service.create_alert(input).await.unwrap();
        assert_eq!(resp.alert.identifier().as_str(), "alert-123");
    }

    #[tokio::test]
    async fn create_alert_propagates_error() {
        let service = build_service::<FailingDb>();
        let input = build_test_input();
        let result = service.create_alert(input).await;
        assert!(matches!(
            result,
            Err(CreateAlertError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn create_alert_rejects_invalid_input() {
        let service = build_service::<MockDb>();
        let mut input = build_test_input();
        input.sender = "Invalid Sender".to_string();
        let result = service.create_alert(input).await;
        assert!(matches!(result, Err(CreateAlertError::ValidationError(_))));
    }

    #[tokio::test]
    async fn update_alert_returns_port_data() {
        let service = build_service::<MockDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let input = build_test_update_input();
        let resp = service.update_alert(identifier, input).await.unwrap();
        assert_eq!(resp.alert.identifier().as_str(), "alert-123");
        assert_eq!(resp.alert.sender().as_str(), "Sender456");
    }

    #[tokio::test]
    async fn update_alert_rejects_invalid_input() {
        let service = build_service::<MockDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let mut input = build_test_update_input();
        input.sender = "Invalid Sender".to_string();
        let result = service.update_alert(identifier, input).await;
        assert!(matches!(result, Err(UpdateAlertError::ValidationError(_))));
    }

    #[tokio::test]
    async fn update_alert_propagates_error() {
        let service = build_service::<FailingDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let input = build_test_update_input();
        let result = service.update_alert(identifier, input).await;
        assert!(matches!(
            result,
            Err(UpdateAlertError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn patch_alert_returns_merged_data() {
        let service = build_service::<MockDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let input = PatchAlertInput {
            sender: Some(Some("PatchedSender".to_string())),
            sent: None,
            status: None,
            msg_type: None,
            source: None,
            scope: None,
            references: None,
        };
        let resp = service.patch_alert(identifier, input).await.unwrap();
        assert_eq!(resp.alert.identifier().as_str(), "alert-123");
        assert_eq!(resp.alert.sender().as_str(), "PatchedSender");
        assert_eq!(resp.alert.status().as_str(), "Actual");
    }

    #[tokio::test]
    async fn patch_alert_rejects_invalid_input() {
        let service = build_service::<MockDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let input = PatchAlertInput {
            sender: Some(Some("Invalid Sender".to_string())),
            sent: None,
            status: None,
            msg_type: None,
            source: None,
            scope: None,
            references: None,
        };
        let result = service.patch_alert(identifier, input).await;
        assert!(matches!(result, Err(PatchAlertError::ValidationError(_))));
    }

    #[tokio::test]
    async fn patch_alert_rejects_null_required_field() {
        let service = build_service::<MockDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let input = PatchAlertInput {
            sender: Some(None),
            sent: None,
            status: None,
            msg_type: None,
            source: None,
            scope: None,
            references: None,
        };
        let result = service.patch_alert(identifier, input).await;
        assert!(matches!(
            result,
            Err(PatchAlertError::ValidationError(msg))
                if msg == "field `sender` cannot be null"
        ));
    }

    #[tokio::test]
    async fn patch_alert_clears_optional_fields() {
        let service = build_service::<MockDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let input = PatchAlertInput {
            sender: None,
            sent: None,
            status: None,
            msg_type: None,
            source: Some(None),
            scope: None,
            references: Some(None),
        };
        let resp = service.patch_alert(identifier, input).await.unwrap();
        assert!(resp.alert.source().as_opt_str().is_none());
        assert_eq!(resp.alert.references().as_db_string(), None);
    }

    #[tokio::test]
    async fn patch_alert_propagates_error() {
        let service = build_service::<FailingDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let input = PatchAlertInput {
            sender: None,
            sent: None,
            status: None,
            msg_type: None,
            source: None,
            scope: None,
            references: None,
        };
        let result = service.patch_alert(identifier, input).await;
        assert!(matches!(
            result,
            Err(PatchAlertError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn delete_alert_returns_port_data() {
        let service = build_service::<MockDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let resp = service.delete_alert(identifier).await;
        assert!(matches!(resp, Ok(DeleteAlertResponse)));
    }

    #[tokio::test]
    async fn delete_alert_propagates_error() {
        let service = build_service::<FailingDb>();
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let result = service.delete_alert(identifier).await;
        assert!(matches!(
            result,
            Err(DeleteAlertError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    fn build_test_input() -> CreateAlertInput {
        CreateAlertInput {
            identifier: "alert-123".to_string(),
            sender: "Sender123".to_string(),
            sent: "2002-05-24T16:49:00-00:00".to_string(),
            status: "Actual".to_string(),
            msg_type: "Alert".to_string(),
            source: Some("Weather Station 1".to_string()),
            scope: "Public".to_string(),
            references: vec!["Sender1,Alert123,2024-06-01T12:00:00-00:00".to_string()],
        }
    }

    fn build_test_update_input() -> UpdateAlertInput {
        UpdateAlertInput {
            sender: "Sender456".to_string(),
            sent: "2003-01-01T12:00:00-00:00".to_string(),
            status: "Test".to_string(),
            msg_type: "Alert".to_string(),
            source: Some("Weather Station 2".to_string()),
            scope: "Public".to_string(),
            references: vec!["Sender2,Alert456,2024-06-02T12:00:00-00:00".to_string()],
        }
    }
}
