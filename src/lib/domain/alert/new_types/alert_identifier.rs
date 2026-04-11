use thiserror::Error;

/// The identifier of the alert message (REQUIRED)
/// (1) A number or string uniquely identifying this message, assigned by the sender.
/// (2) MUST NOT include spaces, commas or restricted characters (< and &).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlertIdentifier(String);

#[derive(Error, Debug)]
pub enum AlertIdentifierError {
    #[error("alert identifier cannot be empty")]
    EmptyIdentifier,
    #[error("alert identifier contains invalid characters")]
    InvalidFormat,
}

impl AlertIdentifier {
    pub fn new(identifier: String) -> Result<Self, AlertIdentifierError> {
        let trimmed = identifier.trim();
        // Validate according to the rules
        if trimmed.is_empty() {
            return Err(AlertIdentifierError::EmptyIdentifier);
        }
        if trimmed.contains(&[' ', ',', '<', '&'][..]) {
            return Err(AlertIdentifierError::InvalidFormat);
        }
        Ok(AlertIdentifier(identifier))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alert_identifier_new() {
        assert_eq!(
            AlertIdentifier::new("Alert123".to_string()).unwrap(),
            AlertIdentifier("Alert123".to_string())
        );
        assert!(matches!(
            AlertIdentifier::new("".to_string()).err().unwrap(),
            AlertIdentifierError::EmptyIdentifier
        ));
        assert!(matches!(
            AlertIdentifier::new("Invalid Identifier".to_string())
                .err()
                .unwrap(),
            AlertIdentifierError::InvalidFormat
        ));
    }
}
