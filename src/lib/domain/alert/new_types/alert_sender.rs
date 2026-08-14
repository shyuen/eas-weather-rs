use thiserror::Error;

/// The identifier of the sender of the alert message (REQUIRED)
/// (1) Identifies the originator of this alert. Guaranteed by assigner to be unique globally; e.g., may be based on an Internet domain name.
/// (2) MUST NOT include spaces, commas or restricted characters (< and &).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct AlertSender(String);

#[derive(Error, Debug)]
pub enum AlertSenderError {
    #[error("alert sender cannot be empty")]
    EmptySender,
    #[error("alert sender contains invalid characters")]
    InvalidFormat,
}

impl AlertSender {
    pub fn new(sender: String) -> Result<Self, AlertSenderError> {
        let trimmed = sender.trim();
        // Validate according to the rules
        if trimmed.is_empty() {
            return Err(AlertSenderError::EmptySender);
        }
        if trimmed.contains(&[' ', ',', '<', '&'][..]) {
            return Err(AlertSenderError::InvalidFormat);
        }
        Ok(AlertSender(sender))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alert_sender_new() {
        assert_eq!(
            AlertSender::new("Sender123".to_string()).unwrap(),
            AlertSender("Sender123".to_string())
        );
        assert!(matches!(
            AlertSender::new("".to_string()).err().unwrap(),
            AlertSenderError::EmptySender
        ));
        assert!(matches!(
            AlertSender::new("Invalid Sender".to_string())
                .err()
                .unwrap(),
            AlertSenderError::InvalidFormat
        ));
    }
}
