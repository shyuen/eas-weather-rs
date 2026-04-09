use serde_derive::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WebserverBasePath(Option<String>);

impl fmt::Display for WebserverBasePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(path) => write!(f, "\"{}\"", path),
            None => write!(f, "None"),
        }
    }
}

#[derive(Error, Debug)]
pub enum WebserverBasePathError {}

impl WebserverBasePath {
    pub fn new(raw_base_path: &str) -> Result<Self, WebserverBasePathError> {
        let trimmed_hostname = raw_base_path.trim();
        if trimmed_hostname.is_empty() {
            return Ok(WebserverBasePath(None));
        }
        // Additional validation can be added here

        Ok(WebserverBasePath(Some(trimmed_hostname.to_string())))
    }

    pub fn default() -> Self {
        WebserverBasePath(None)
    }

    pub fn get(&self) -> &Option<String> {
        &self.0
    }
}
