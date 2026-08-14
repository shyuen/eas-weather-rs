use crate::domain::alert::model::{Alert, CreateAlertInput};
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
    AlertPort, CreateAlertError, CreateAlertResponse, GetDailyAlertsError, GetDailyAlertsResponse,
    GetLatestAlertsError, GetLatestAlertsResponse,
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
    pub fn new(db_serv: DatabaseService<D>) -> Self {
        let db_port = db_serv.get_port();

        // Log the initialization of the AlertService
        info!("Initializing AlertService");

        Self {
            db_port: db_port.clone(),
        }
    }

    /// Get the Database repository
    pub fn get_db_port(&self) -> &D {
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
        AlertService::new(db_service)
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
}
