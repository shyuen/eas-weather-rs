use crate::domain::utils::helpers::capitalize_first_lowercase_rest;

use thiserror::Error;

/// The code denoting the intended distribution of the alert message (REQUIRED)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlertScope(AlertScopeValue);

/// Valid Code Values:
///     “Public” - For general dissemination to unrestricted audiences
///     “Restricted” - For dissemination only to users with a known operational requirement (see <restriction>, below)
///     “Private” - For dissemination only to specified addresses (see <addresses>, below)
#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum AlertScopeValue {
    Public,
    Restricted,
    Private,
}

#[derive(Error, Debug)]
pub enum AlertScopeError {
    #[error("alert scope cannot be empty")]
    EmptyScope,
    #[error("invalid alert scope value")]
    InvalidScopeValue,
}

impl AlertScope {
    pub fn new(scope: String) -> Result<Self, AlertScopeError> {
        // Trim whitespace
        let trimmed = scope.trim();
        // Make all text lowercase for case-insensitive comparison
        let lowered = trimmed.to_lowercase();

        // Take the first character, convert it to uppercase, and chain the rest of the characters
        let sanitized = capitalize_first_lowercase_rest(&lowered);

        match sanitized.as_str() {
            "Public" => Ok(AlertScope(AlertScopeValue::Public)),
            "Restricted" => Ok(AlertScope(AlertScopeValue::Restricted)),
            "Private" => Ok(AlertScope(AlertScopeValue::Private)),
            "" => Err(AlertScopeError::EmptyScope),
            _ => Err(AlertScopeError::InvalidScopeValue),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alert_scope_new() {
        assert_eq!(
            AlertScope::new("Public".to_string()).unwrap(),
            AlertScope(AlertScopeValue::Public)
        );
        assert_eq!(
            AlertScope::new("restricted".to_string()).unwrap(),
            AlertScope(AlertScopeValue::Restricted)
        );
        assert_eq!(
            AlertScope::new(" PRIVATE ".to_string()).unwrap(),
            AlertScope(AlertScopeValue::Private)
        );
        assert!(matches!(
            AlertScope::new("".to_string()).err().unwrap(),
            AlertScopeError::EmptyScope
        ));
        assert!(matches!(
            AlertScope::new("InvalidScope".to_string()).err().unwrap(),
            AlertScopeError::InvalidScopeValue
        ));
    }
}
