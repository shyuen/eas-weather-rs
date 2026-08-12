use serde_derive::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deserialize, Serialize)]
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

    pub fn get(&self) -> &Option<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_path() {
        assert_eq!(
            WebserverBasePath::new("/api").unwrap(),
            WebserverBasePath(Some("/api".to_string()))
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            WebserverBasePath::new(" /api ").unwrap(),
            WebserverBasePath(Some("/api".to_string()))
        );
    }

    #[test]
    fn empty_path_becomes_none() {
        assert_eq!(WebserverBasePath::new("").unwrap(), WebserverBasePath(None));
        assert_eq!(
            WebserverBasePath::new("  ").unwrap(),
            WebserverBasePath(None)
        );
    }

    #[test]
    fn defaults_to_none() {
        assert_eq!(WebserverBasePath::default(), WebserverBasePath(None));
    }
}
