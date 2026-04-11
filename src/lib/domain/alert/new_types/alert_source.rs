use thiserror::Error;

/// The text identifying the source of the alert message (OPTIONAL)
/// The particular source of this alert; e.g., an operator or a specific device.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlertSource(Option<String>);

#[derive(Error, Debug)]
pub enum AlertSourceError {}

impl AlertSource {
    pub fn new(source: &str) -> Result<Self, AlertSourceError> {
        let trimmed = source.trim();
        if trimmed.is_empty() {
            Ok(AlertSource(None))
        } else {
            Ok(AlertSource(Some(trimmed.to_string())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alert_source_new() {
        assert_eq!(
            AlertSource::new("Weather Station 1").unwrap(),
            AlertSource(Some("Weather Station 1".to_string()))
        );
        assert_eq!(AlertSource::new("   ").unwrap(), AlertSource(None));
    }
}
