use serde_derive::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WebserverHostname(String);

impl fmt::Display for WebserverHostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

#[derive(Error, Debug)]
pub enum WebserverHostnameError {
    #[error("webserver hostname cannot be empty")]
    EmptyHostname,
}

impl WebserverHostname {
    pub fn new(raw_hostname: &str) -> Result<Self, WebserverHostnameError> {
        let trimmed_hostname = raw_hostname.trim();
        if trimmed_hostname.is_empty() {
            return Err(WebserverHostnameError::EmptyHostname);
        }
        // Additional validation can be added here (e.g., valid domain name or IP address)

        Ok(WebserverHostname(trimmed_hostname.to_string()))
    }

    pub fn get(&self) -> &String {
        &self.0
    }
}

impl Default for WebserverHostname {
    fn default() -> Self {
        WebserverHostname("localhost".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_hostname() {
        assert_eq!(
            WebserverHostname::new("localhost").unwrap(),
            WebserverHostname("localhost".to_string())
        );
        assert_eq!(
            WebserverHostname::new("0.0.0.0").unwrap(),
            WebserverHostname("0.0.0.0".to_string())
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            WebserverHostname::new("  example.com ").unwrap(),
            WebserverHostname("example.com".to_string())
        );
    }

    #[test]
    fn rejects_empty_hostname() {
        assert!(matches!(
            WebserverHostname::new("").err().unwrap(),
            WebserverHostnameError::EmptyHostname
        ));
        assert!(matches!(
            WebserverHostname::new("   ").err().unwrap(),
            WebserverHostnameError::EmptyHostname
        ));
    }

    #[test]
    fn defaults_to_localhost() {
        assert_eq!(
            WebserverHostname::default(),
            WebserverHostname("localhost".to_string())
        );
    }
}
