use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::new_types::alert_msg_type::AlertMsgType;
use crate::domain::alert::new_types::alert_references::AlertReferences;
use crate::domain::alert::new_types::alert_scope::AlertScope;
use crate::domain::alert::new_types::alert_sender::AlertSender;
use crate::domain::alert::new_types::alert_sent::AlertSent;
use crate::domain::alert::new_types::alert_source::AlertSource;
use crate::domain::alert::new_types::alert_status::AlertStatus;

/// Raw, unvalidated fields for creating an alert. The domain service validates
/// these into an [`Alert`], so this type is the input contract from the HTTP
/// layer without dragging adaptor types into the domain.
#[derive(Debug, Clone)]
pub struct CreateAlertInput {
    pub identifier: String,
    pub sender: String,
    pub sent: String,
    pub status: String,
    pub msg_type: String,
    pub source: Option<String>,
    pub scope: String,
    pub references: Vec<String>,
}

/// Complete alert structure.
/// See https://docs.oasis-open.org/emergency/cap/v1.2/CAP-v1.2.html for reference.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Alert {
    identifier: AlertIdentifier,
    sender: AlertSender,
    sent: AlertSent,
    status: AlertStatus,
    msg_type: AlertMsgType,
    source: AlertSource,
    scope: AlertScope,
    references: AlertReferences,
}

impl Alert {
    /// Constructs an [`Alert`] from its eight required CAP fields.
    ///
    /// These form the fixed CAP 1.2 alert structure; each field is a distinct
    /// validated newtype, so a builder or params struct would add indirection
    /// without reducing real complexity.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identifier: AlertIdentifier,
        sender: AlertSender,
        sent: AlertSent,
        status: AlertStatus,
        msg_type: AlertMsgType,
        source: AlertSource,
        scope: AlertScope,
        references: AlertReferences,
    ) -> Self {
        Alert {
            identifier,
            sender,
            sent,
            status,
            msg_type,
            source,
            scope,
            references,
        }
    }

    pub fn identifier(&self) -> &AlertIdentifier {
        &self.identifier
    }
    pub fn sender(&self) -> &AlertSender {
        &self.sender
    }
    pub fn sent(&self) -> &AlertSent {
        &self.sent
    }
    pub fn status(&self) -> &AlertStatus {
        &self.status
    }
    pub fn msg_type(&self) -> &AlertMsgType {
        &self.msg_type
    }
    pub fn source(&self) -> &AlertSource {
        &self.source
    }
    pub fn scope(&self) -> &AlertScope {
        &self.scope
    }
    pub fn references(&self) -> &AlertReferences {
        &self.references
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    use crate::domain::alert::new_types::alert_references::ExtendedMessageIdentifier;

    #[test]
    fn test_alert_new() {
        let identifier = AlertIdentifier::new("alert-123".to_string()).unwrap();
        let sender = AlertSender::new("Sender123".to_string()).unwrap();
        let sent = AlertSent::new(OffsetDateTime::now_utc()).unwrap();
        let status = AlertStatus::new("Actual".to_string()).unwrap();
        let msg_type = AlertMsgType::new("Alert".to_string()).unwrap();
        let source = AlertSource::new("Weather Station 1").unwrap();
        let scope = AlertScope::new("Public".to_string()).unwrap();
        let reference =
            ExtendedMessageIdentifier::new("Sender1,Alert123,2024-06-01T12:00:00-00:00").unwrap();
        let references = AlertReferences::new(vec![reference.clone()]).unwrap();

        let alert = Alert::new(
            identifier, sender, sent, status, msg_type, source, scope, references,
        );

        // Verify the constructed alert carries the expected fields via its Debug
        // representation (no public getters exist yet).
        let debug = format!("{:?}", alert);
        assert!(debug.contains("alert-123"));
        assert!(debug.contains("Sender123"));
        assert!(debug.contains("Actual"));
        assert!(debug.contains("Weather Station 1"));
        assert!(debug.contains("Public"));
    }
}
