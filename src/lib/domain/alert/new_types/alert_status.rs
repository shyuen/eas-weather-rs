use thiserror::Error;

use crate::domain::utils::helpers::capitalize_first_lowercase_rest;

/// The code denoting the appropriate handling of the alert status (REQUIRED)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct AlertStatus(AlertStatusValue);

/// Valid Code Values:
///     “Actual” - Actionable by all targeted recipients
///     “Exercise” - Actionable only by designated exercise participants; exercise identifier SHOULD appear in <note>
///     “System” - For messages that support alert network internal functions
///     “Test” - Technical testing only, all recipients disregard
///     “Draft” – A preliminary template or draft, not actionable in its current form
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum AlertStatusValue {
    Actual,
    Exercise,
    System,
    Test,
    Draft,
}

#[derive(Error, Debug)]
pub enum AlertStatusError {
    #[error("invalid alert status value")]
    InvalidStatusValue,
    #[error("alert status value cannot be empty")]
    EmptyStatusValue,
}

impl AlertStatus {
    pub fn new(status: String) -> Result<Self, AlertStatusError> {
        // Trim whitespace
        let trimmed = status.trim();
        // Make all text lowercase for case-insensitive comparison
        let lowered = trimmed.to_lowercase();

        // Take the first character, convert it to uppercase, and chain the rest of the characters
        let sanitized = capitalize_first_lowercase_rest(&lowered);

        match sanitized.as_str() {
            "Actual" => Ok(AlertStatus(AlertStatusValue::Actual)),
            "Exercise" => Ok(AlertStatus(AlertStatusValue::Exercise)),
            "System" => Ok(AlertStatus(AlertStatusValue::System)),
            "Test" => Ok(AlertStatus(AlertStatusValue::Test)),
            "Draft" => Ok(AlertStatus(AlertStatusValue::Draft)),
            "" => Err(AlertStatusError::EmptyStatusValue),
            _ => Err(AlertStatusError::InvalidStatusValue),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alert_status_new() {
        assert_eq!(
            AlertStatus::new("Actual".to_string()).unwrap(),
            AlertStatus(AlertStatusValue::Actual)
        );
        assert_eq!(
            AlertStatus::new("exercise".to_string()).unwrap(),
            AlertStatus(AlertStatusValue::Exercise)
        );
        assert_eq!(
            AlertStatus::new(" SYSTEM ".to_string()).unwrap(),
            AlertStatus(AlertStatusValue::System)
        );
        assert_eq!(
            AlertStatus::new("TeSt".to_string()).unwrap(),
            AlertStatus(AlertStatusValue::Test)
        );
        assert_eq!(
            AlertStatus::new("draft".to_string()).unwrap(),
            AlertStatus(AlertStatusValue::Draft)
        );
        assert!(matches!(
            AlertStatus::new("".to_string()).err().unwrap(),
            AlertStatusError::EmptyStatusValue
        ));
        assert!(matches!(
            AlertStatus::new("InvalidStatus".to_string()).err().unwrap(),
            AlertStatusError::InvalidStatusValue
        ));
    }
}
