use thiserror::Error;

/// A single reference to an earlier alert message within the <references> group (OPTIONAL)
/// The extended message identifier is in the form (sender,identifier,sent).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct ExtendedMessageIdentifier(String);

#[derive(Error, Debug)]
pub enum ExtendedMessageIdentifierError {
    #[error("alert reference cannot be empty")]
    EmptyReference,
    #[error("alert reference needs to be in the form `sender,identifier,sent`")]
    InvalidFormat,
}

impl ExtendedMessageIdentifier {
    pub fn new(emi: &str) -> Result<Self, ExtendedMessageIdentifierError> {
        let trimmed = emi.trim();
        if trimmed.is_empty() {
            return Err(ExtendedMessageIdentifierError::EmptyReference);
        }
        // Additional validation can be added here if needed
        // Check if it contains the required format `sender,identifier,sent`
        let parts: Vec<&str> = trimmed.split(',').collect();

        // There should be exactly 3 parts
        if parts.len() != 3 {
            return Err(ExtendedMessageIdentifierError::InvalidFormat);
        }
        // Check if each part is non-empty after trimming
        for part in parts {
            if part.trim().is_empty() {
                return Err(ExtendedMessageIdentifierError::InvalidFormat);
            }
        }
        Ok(ExtendedMessageIdentifier(trimmed.to_string()))
    }
}

/// The group listing identifying earlier message(s) referenced by the alert message (OPTIONAL)
/// (1) The extended message identifier(s) (in the form sender,identifier,sent) of an earlier CAP message or messages referenced by this one.
/// (2) If multiple messages are referenced, they SHALL be separated by whitespace.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct AlertReferences(Option<Vec<ExtendedMessageIdentifier>>);

#[derive(Error, Debug)]
pub enum AlertReferencesError {}

impl AlertReferences {
    pub fn new(references: Vec<ExtendedMessageIdentifier>) -> Result<Self, AlertReferencesError> {
        if references.is_empty() {
            Ok(AlertReferences(None))
        } else {
            Ok(AlertReferences(Some(references)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extended_message_identifier_new() {
        assert_eq!(
            ExtendedMessageIdentifier::new("sender1,Alert123,2024-06-01T12:00:00Z").unwrap(),
            ExtendedMessageIdentifier("sender1,Alert123,2024-06-01T12:00:00Z".to_string())
        );
        assert!(matches!(
            ExtendedMessageIdentifier::new("").err().unwrap(),
            ExtendedMessageIdentifierError::EmptyReference
        ));
        assert!(matches!(
            ExtendedMessageIdentifier::new("InvalidFormat")
                .err()
                .unwrap(),
            ExtendedMessageIdentifierError::InvalidFormat
        ));
        assert!(matches!(
            ExtendedMessageIdentifier::new("sender1,,2024-06-01T12:00:00Z")
                .err()
                .unwrap(),
            ExtendedMessageIdentifierError::InvalidFormat
        ));
    }

    #[test]
    fn test_alert_references_new() {
        let emi =
            ExtendedMessageIdentifier::new("Sender1,Alert123,2024-06-01T12:00:00-00:00").unwrap();
        let references = vec![emi.clone()];
        assert_eq!(
            AlertReferences::new(references.clone()).unwrap(),
            AlertReferences(Some(vec![emi]))
        );
        assert_eq!(AlertReferences::new(vec![]).unwrap(), AlertReferences(None));
    }
}
