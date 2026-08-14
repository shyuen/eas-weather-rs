use crate::domain::utils::helpers::capitalize_first_lowercase_rest;

use thiserror::Error;

/// The code denoting the nature of the alert message (REQUIRED)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct AlertMsgType(AlertMsgTypeValue);

/// Valid Code Values:
///     “Alert” - Initial information requiring attention by targeted recipients
///     “Update” - Updates and supercedes the earlier message(s) identified in <references>
///     “Cancel” - Cancels the earlier message(s) identified in <references>
///     “Ack” - Acknowledges receipt and acceptance of the message(s) identified in <references>
///     “Error” - Indicates rejection of the message(s) identified in <references>; explanation SHOULD appear in <note>
#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, serde::Serialize)]
pub enum AlertMsgTypeValue {
    Alert,
    Update,
    Cancel,
    Ack,
    Error,
}

#[derive(Error, Debug)]
pub enum AlertMsgTypeError {
    #[error("alert message type cannot be empty")]
    EmptyMsgType,
    #[error("invalid alert message type value")]
    InvalidMsgTypeValue,
}

impl AlertMsgType {
    pub fn new(msg_type: String) -> Result<Self, AlertMsgTypeError> {
        // Trim whitespace
        let trimmed = msg_type.trim();
        // Make all text lowercase for case-insensitive comparison
        let lowered = trimmed.to_lowercase();

        // Take the first character, convert it to uppercase, and chain the rest of the characters
        let sanitized = capitalize_first_lowercase_rest(&lowered);

        match sanitized.as_str() {
            "Alert" => Ok(AlertMsgType(AlertMsgTypeValue::Alert)),
            "Update" => Ok(AlertMsgType(AlertMsgTypeValue::Update)),
            "Cancel" => Ok(AlertMsgType(AlertMsgTypeValue::Cancel)),
            "Ack" => Ok(AlertMsgType(AlertMsgTypeValue::Ack)),
            "Error" => Ok(AlertMsgType(AlertMsgTypeValue::Error)),
            "" => Err(AlertMsgTypeError::EmptyMsgType),
            _ => Err(AlertMsgTypeError::InvalidMsgTypeValue),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alert_msg_type_new() {
        assert_eq!(
            AlertMsgType::new("Alert".to_string()).unwrap(),
            AlertMsgType(AlertMsgTypeValue::Alert)
        );
        assert_eq!(
            AlertMsgType::new("update".to_string()).unwrap(),
            AlertMsgType(AlertMsgTypeValue::Update)
        );
        assert_eq!(
            AlertMsgType::new("CANCEL".to_string()).unwrap(),
            AlertMsgType(AlertMsgTypeValue::Cancel)
        );
        assert_eq!(
            AlertMsgType::new("Ack".to_string()).unwrap(),
            AlertMsgType(AlertMsgTypeValue::Ack)
        );
        assert_eq!(
            AlertMsgType::new("error".to_string()).unwrap(),
            AlertMsgType(AlertMsgTypeValue::Error)
        );
        assert!(matches!(
            AlertMsgType::new("".to_string()).err().unwrap(),
            AlertMsgTypeError::EmptyMsgType
        ));
        assert!(matches!(
            AlertMsgType::new("InvalidType".to_string()).err().unwrap(),
            AlertMsgTypeError::InvalidMsgTypeValue
        ));
    }
}
