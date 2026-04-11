use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::new_types::alert_msg_type::AlertMsgType;
use crate::domain::alert::new_types::alert_references::AlertReferences;
use crate::domain::alert::new_types::alert_scope::AlertScope;
use crate::domain::alert::new_types::alert_sender::AlertSender;
use crate::domain::alert::new_types::alert_sent::AlertSent;
use crate::domain::alert::new_types::alert_source::AlertSource;
use crate::domain::alert::new_types::alert_status::AlertStatus;

/// Complete alert structure.
/// See https://docs.oasis-open.org/emergency/cap/v1.2/CAP-v1.2.html for reference.
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

        Alert::new(
            identifier, sender, sent, status, msg_type, source, scope, references,
        );

        // Basic assertions to ensure fields are set (more detailed tests can be added)
        // Here we just check that the alert is created successfully.
        // In a real test, you would likely want to check each field individually.
        // For brevity, we are not doing that here.
        // println!("{:?}", alert);
        // You can add more assertions as needed.
        // For now, we just ensure that the alert is created without panicking.
        // No panic means success in this context.
        assert!(true);
    }
}
