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

    pub fn default() -> Self {
        WebserverHostname("localhost".to_string())
    }

    pub fn get(&self) -> &String {
        &self.0
    }
}
