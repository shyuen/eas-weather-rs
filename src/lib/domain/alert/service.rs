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
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::field::Empty;
use tracing::{error, info};

/// The AlertService struct provides methods for managing alerts.
/// Contains ports to interact with other services
#[derive(Debug, Clone)]
pub struct AlertService<AP>
where
    AP: AlertPort,
{
    alert_port: AP,
}

impl<AP> AlertService<AP>
where
    AP: AlertPort,
{
    /// Creates a new instance of AlertService.
    pub fn new(alert_port: &AP) -> Self {
        info!(event_kind = "service", "Initializing AlertService");
        Self {
            alert_port: alert_port.clone(),
        }
    }

    /// Retrieve the latest alerts sent within the last 24 hours from the database.
    #[tracing::instrument(skip(self), fields(operation = "get_daily_alerts", limit, offset, result = Empty), level = "debug")]
    pub async fn get_daily_alerts(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<GetDailyAlertsResponse, GetDailyAlertsError> {
        let span = tracing::Span::current();
        match self.alert_port.get_daily_alerts_data(limit, offset).await {
            Ok(resp) => {
                span.record("result", "ok");
                info!(
                    event_kind = "service",
                    "get_daily_alerts returned {} of {} alerts",
                    resp.alerts.len(),
                    resp.total
                );
                Ok(resp)
            }
            Err(err) => {
                error!(event_kind = "service", error_code = err.code(), message = %err, "get_daily_alerts failed");
                Err(err)
            }
        }
    }

    /// Retrieve the latest alerts from the database.
    #[tracing::instrument(skip(self), fields(operation = "get_latest_alerts", limit, offset, result = Empty), level = "debug")]
    pub async fn get_latest_alerts(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<GetLatestAlertsResponse, GetLatestAlertsError> {
        let span = tracing::Span::current();
        match self.alert_port.get_latest_alerts_data(limit, offset).await {
            Ok(resp) => {
                span.record("result", "ok");
                info!(
                    event_kind = "service",
                    "get_latest_alerts returned {} of {} alerts",
                    resp.alerts.len(),
                    resp.total
                );
                Ok(resp)
            }
            Err(err) => {
                error!(event_kind = "service", error_code = err.code(), message = %err, "get_latest_alerts failed");
                Err(err)
            }
        }
    }

    /// Validate and persist a new alert to the database.
    #[tracing::instrument(skip(self, input), fields(operation = "create_alert", result = Empty), level = "debug")]
    pub async fn create_alert(
        &self,
        input: CreateAlertInput,
    ) -> Result<CreateAlertResponse, CreateAlertError> {
        let span = tracing::Span::current();
        let alert = match build_alert(input) {
            Ok(alert) => alert,
            Err(err) => {
                error!(event_kind = "service", error_code = "validation_error", message = %err, "create_alert rejected invalid input");
                return Err(CreateAlertError::ValidationError(err));
            }
        };

        match self.alert_port.create_alert_data(alert).await {
            Ok(resp) => {
                span.record("result", "ok");
                info!(
                    event_kind = "service",
                    "create_alert persisted successfully"
                );
                Ok(resp)
            }
            Err(err) => {
                error!(event_kind = "service", error_code = err.code(), message = %err, "create_alert failed");
                Err(err)
            }
        }
    }

    /// Validate and replace an existing alert (identified by `identifier`).
    #[tracing::instrument(skip(self, input), fields(operation = "update_alert", identifier = tracing::field::Empty, result = Empty), level = "debug")]
    pub async fn update_alert(
        &self,
        identifier: AlertIdentifier,
        input: UpdateAlertInput,
    ) -> Result<UpdateAlertResponse, UpdateAlertError> {
        let span = tracing::Span::current();
        span.record("identifier", identifier.as_str());
        let alert = match build_alert_for_update(identifier, input) {
            Ok(alert) => alert,
            Err(err) => {
                error!(event_kind = "service", error_code = "validation_error", message = %err, "update_alert rejected invalid input");
                return Err(UpdateAlertError::ValidationError(err));
            }
        };

        let alert_identifier = alert.identifier().clone();
        match self
            .alert_port
            .update_alert_data(&alert_identifier, alert)
            .await
        {
            Ok(resp) => {
                span.record("result", "ok");
                info!(
                    event_kind = "service",
                    "update_alert persisted successfully"
                );
                Ok(resp)
            }
            Err(err) => {
                error!(event_kind = "service", error_code = err.code(), message = %err, "update_alert failed");
                Err(err)
            }
        }
    }

    /// Validate and partially update an existing alert (identified by
    /// `identifier`). Fields present in `input` override the stored value;
    /// fields absent are left untouched.
    #[tracing::instrument(skip(self, input), fields(operation = "patch_alert", identifier = tracing::field::Empty, result = Empty), level = "debug")]
    pub async fn patch_alert(
        &self,
        identifier: AlertIdentifier,
        input: PatchAlertInput,
    ) -> Result<PatchAlertResponse, PatchAlertError> {
        let span = tracing::Span::current();
        span.record("identifier", identifier.as_str());
        let existing = match self.alert_port.get_alert_data(&identifier).await {
            Ok(resp) => resp.alert,
            Err(err) => {
                let patch_err = match err {
                    GetAlertError::NotFound => PatchAlertError::NotFound,
                    GetAlertError::DatabaseError(msg) => PatchAlertError::DatabaseError(msg),
                    GetAlertError::DatabaseConnectionError(msg) => {
                        PatchAlertError::DatabaseConnectionError(msg)
                    }
                    GetAlertError::DataConversionError(msg) => {
                        PatchAlertError::DataConversionError(msg)
                    }
                };
                error!(event_kind = "service", error_code = patch_err.code(), message = %patch_err, "patch_alert failed to fetch existing alert");
                return Err(patch_err);
            }
        };

        let alert = match build_alert_for_patch(identifier, &existing, input) {
            Ok(alert) => alert,
            Err(err) => {
                error!(event_kind = "service", error_code = "validation_error", message = %err, "patch_alert rejected invalid input");
                return Err(PatchAlertError::ValidationError(err));
            }
        };

        let alert_identifier = alert.identifier().clone();
        match self
            .alert_port
            .update_alert_data(&alert_identifier, alert)
            .await
        {
            Ok(resp) => {
                span.record("result", "ok");
                info!(event_kind = "service", "patch_alert persisted successfully");
                Ok(PatchAlertResponse { alert: resp.alert })
            }
            Err(err) => {
                let patch_err = match err {
                    UpdateAlertError::NotFound => PatchAlertError::NotFound,
                    UpdateAlertError::DatabaseError(msg) => PatchAlertError::DatabaseError(msg),
                    UpdateAlertError::DatabaseConnectionError(msg) => {
                        PatchAlertError::DatabaseConnectionError(msg)
                    }
                    UpdateAlertError::ValidationError(msg) => PatchAlertError::ValidationError(msg),
                };
                error!(event_kind = "service", error_code = patch_err.code(), message = %patch_err, "patch_alert failed");
                Err(patch_err)
            }
        }
    }

    /// Delete an existing alert (identified by `identifier`).
    #[tracing::instrument(skip(self), fields(operation = "delete_alert", identifier = tracing::field::Empty, result = Empty), level = "debug")]
    pub async fn delete_alert(
        &self,
        identifier: AlertIdentifier,
    ) -> Result<DeleteAlertResponse, DeleteAlertError> {
        let span = tracing::Span::current();
        span.record("identifier", identifier.as_str());
        match self.alert_port.delete_alert_data(&identifier).await {
            Ok(resp) => {
                span.record("result", "ok");
                info!(event_kind = "service", "delete_alert removed successfully");
                Ok(resp)
            }
            Err(err) => {
                error!(event_kind = "service", error_code = err.code(), message = %err, "delete_alert failed");
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
    use crate::test_support::{FailingDb, MockDb};

    /// Build an `AlertService<AP>` over the supplied `AlertPort` double.
    fn build_service<AP>(d: &AP) -> AlertService<AP>
    where
        AP: AlertPort,
    {
        AlertService::new(d)
    }

    #[tokio::test]
    async fn get_latest_alerts_returns_port_data() {
        let service = build_service(&MockDb);
        let resp = service.get_latest_alerts(10, 5).await.unwrap();
        assert_eq!(resp.total, 42);
        assert!(resp.alerts.is_empty());
    }

    #[tokio::test]
    async fn get_latest_alerts_propagates_error() {
        let service = build_service(&FailingDb);
        let result = service.get_latest_alerts(10, 5).await;
        assert!(matches!(
            result,
            Err(GetLatestAlertsError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn get_daily_alerts_returns_port_data() {
        let service = build_service(&MockDb);
        let resp = service.get_daily_alerts(10, 5).await.unwrap();
        assert_eq!(resp.total, 42);
        assert!(resp.alerts.is_empty());
    }

    #[tokio::test]
    async fn get_daily_alerts_propagates_error() {
        let service = build_service(&FailingDb);
        let result = service.get_daily_alerts(10, 5).await;
        assert!(matches!(
            result,
            Err(GetDailyAlertsError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn create_alert_returns_port_data() {
        let service = build_service(&MockDb);
        let input = build_test_input();
        let resp = service.create_alert(input).await.unwrap();
        assert_eq!(resp.alert.identifier().as_str(), "alert-123");
    }

    #[tokio::test]
    async fn create_alert_propagates_error() {
        let service = build_service(&FailingDb);
        let input = build_test_input();
        let result = service.create_alert(input).await;
        assert!(matches!(
            result,
            Err(CreateAlertError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn create_alert_rejects_invalid_input() {
        let service = build_service(&MockDb);
        let mut input = build_test_input();
        input.sender = "Invalid Sender".to_string();
        let result = service.create_alert(input).await;
        assert!(matches!(result, Err(CreateAlertError::ValidationError(_))));
    }

    #[tokio::test]
    async fn update_alert_returns_port_data() {
        let service = build_service(&MockDb);
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let input = build_test_update_input();
        let resp = service.update_alert(identifier, input).await.unwrap();
        assert_eq!(resp.alert.identifier().as_str(), "alert-123");
        assert_eq!(resp.alert.sender().as_str(), "Sender456");
    }

    #[tokio::test]
    async fn update_alert_rejects_invalid_input() {
        let service = build_service(&MockDb);
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let mut input = build_test_update_input();
        input.sender = "Invalid Sender".to_string();
        let result = service.update_alert(identifier, input).await;
        assert!(matches!(result, Err(UpdateAlertError::ValidationError(_))));
    }

    #[tokio::test]
    async fn update_alert_propagates_error() {
        let service = build_service(&FailingDb);
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
        let service = build_service(&MockDb);
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
        let service = build_service(&MockDb);
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
        let service = build_service(&MockDb);
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
        let service = build_service(&MockDb);
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
        let service = build_service(&FailingDb);
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
        let service = build_service(&MockDb);
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let resp = service.delete_alert(identifier).await;
        assert!(matches!(resp, Ok(DeleteAlertResponse)));
    }

    #[tokio::test]
    async fn delete_alert_propagates_error() {
        let service = build_service(&FailingDb);
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let result = service.delete_alert(identifier).await;
        assert!(matches!(
            result,
            Err(DeleteAlertError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[test]
    fn error_logs_carry_error_code_field() {
        use std::sync::{Arc, Mutex};
        use tracing::field::Visit;
        use tracing::subscriber::with_default;

        #[derive(Default)]
        struct FieldCapture {
            error_codes: Vec<String>,
            event_kinds: Vec<String>,
        }

        impl Visit for FieldCapture {
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                match field.name() {
                    "error_code" => self.error_codes.push(value.to_string()),
                    "event_kind" => self.event_kinds.push(value.to_string()),
                    _ => {}
                }
            }
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "error_code" {
                    self.error_codes.push(format!("{value:?}"));
                }
            }
        }

        struct CapturingSubscriber(Arc<Mutex<(Vec<String>, Vec<String>)>>);

        impl tracing::Subscriber for CapturingSubscriber {
            fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
                true
            }
            fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::Id {
                tracing::Id::from_u64(1)
            }
            fn record(&self, _span: &tracing::Id, _values: &tracing::span::Record<'_>) {}
            fn record_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id) {}
            fn event(&self, event: &tracing::Event<'_>) {
                let mut capture = FieldCapture::default();
                event.record(&mut capture);
                let (codes, kinds) = &mut *self.0.lock().unwrap();
                codes.extend(capture.error_codes);
                kinds.extend(capture.event_kinds);
            }
            fn enter(&self, _span: &tracing::Id) {}
            fn exit(&self, _span: &tracing::Id) {}
        }

        let captured = Arc::new(Mutex::new((Vec::new(), Vec::new())));
        let subscriber = CapturingSubscriber(captured.clone());

        with_default(subscriber, || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let service = rt.block_on(async { build_service(&FailingDb) });
            rt.block_on(async {
                let _ = service.get_latest_alerts(10, 5).await;
                let _ = service.create_alert(build_test_input()).await;
            });
        });

        let (codes, kinds) = &*captured.lock().unwrap();
        assert!(codes.contains(&"database_error".to_string()));
        // Every service-logged event is tagged with the `event_kind = "service"`
        // key so aggregators can distinguish application (service) events from
        // transport (http) events.
        assert!(kinds.iter().all(|k| k == "service"));
        assert!(!kinds.is_empty());
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
