use thiserror::Error;
use time::OffsetDateTime;

/// The time and date of the origination of the alert message (REQUIRED)
/// (1) The date and time SHALL be represented in the DateTime Data Type (See Implementation Notes) format
///     (e.g., "2002-05-24T16:49:00-07:00" for 24 May 2002 at 16:49 PDT).
/// (2) Alphabetic timezone designators such as “Z” MUST NOT be used.
///     The timezone for UTC MUST be represented as “-00:00”
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlertSent(OffsetDateTime);

#[derive(Error, Debug)]
pub enum AlertSentError {}

impl AlertSent {
    pub fn new(timestamp: OffsetDateTime) -> Result<Self, AlertSentError> {
        // Additional validation can be added here if needed
        Ok(AlertSent(timestamp))
    }
}

/// Serialize as an RFC 3339 timestamp string (e.g. "2002-05-24T16:49:00-07:00").
/// `OffsetDateTime` has no plain derive support without the `serde-well-known`
/// feature, so the format is applied explicitly here.
impl serde::Serialize for AlertSent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let formatted = self
            .0
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&formatted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alert_sent_new() {
        let datetime = OffsetDateTime::now_utc();
        assert_eq!(AlertSent::new(datetime).unwrap(), AlertSent(datetime));
    }
}
