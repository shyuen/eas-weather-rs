use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deserialize, Serialize)]
pub struct WebserverApiKey(Option<String>);

impl fmt::Display for WebserverApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(key) => write!(f, "\"{}\"", key.chars().map(|_| '*').collect::<String>()),
            None => write!(f, "None"),
        }
    }
}

#[derive(Error, Debug)]
pub enum WebserverApiKeyError {
    #[error("database connection string could not be loaded from file")]
    BadFileLoad(String),
}

impl WebserverApiKey {
    pub fn new(raw_api_key_file: &str) -> Result<Self, WebserverApiKeyError> {
        let trimmed_raw_file_path = raw_api_key_file.trim().to_string();

        // Check if given file path was empty
        if trimmed_raw_file_path.is_empty() {
            return Ok(WebserverApiKey(None));
        }

        // Read the string from the file
        let file_contents = match fs::read_to_string(trimmed_raw_file_path) {
            Ok(raw_api_key) => raw_api_key,
            Err(e) => {
                // Some issue loading the file
                return Err(WebserverApiKeyError::BadFileLoad(e.to_string()));
            }
        };

        // Check if the connection string was empty
        let trimmed_file_contents = file_contents.trim();
        if trimmed_file_contents.is_empty() {
            return Ok(WebserverApiKey(None));
        }

        // TODO - Other validation of the connection string as neccessary

        Ok(WebserverApiKey(Some(trimmed_file_contents.to_string())))
    }

    pub fn get(&self) -> &Option<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn loads_key_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "supersecretkey").unwrap();
        let key = WebserverApiKey::new(file.path().to_str().unwrap()).unwrap();
        assert_eq!(key.get().as_deref(), Some("supersecretkey"));
    }

    #[test]
    fn trims_file_contents() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "  secret123").unwrap();
        let key = WebserverApiKey::new(file.path().to_str().unwrap()).unwrap();
        assert_eq!(key.get().as_deref(), Some("secret123"));
    }

    #[test]
    fn empty_path_becomes_none() {
        assert_eq!(WebserverApiKey::new("").unwrap(), WebserverApiKey(None));
        assert_eq!(WebserverApiKey::new("   ").unwrap(), WebserverApiKey(None));
    }

    #[test]
    fn empty_file_becomes_none() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "  ").unwrap();
        let key = WebserverApiKey::new(file.path().to_str().unwrap()).unwrap();
        assert_eq!(key, WebserverApiKey(None));
    }

    #[test]
    fn rejects_missing_file() {
        assert!(matches!(
            WebserverApiKey::new("/nonexistent/key.pem").err().unwrap(),
            WebserverApiKeyError::BadFileLoad(_)
        ));
    }

    #[test]
    fn display_masks_key() {
        let key = WebserverApiKey(Some("supersecretkey".to_string()));
        let shown = key.to_string();
        assert!(!shown.contains("supersecretkey"), "key leaked: {shown}");
    }

    #[test]
    fn defaults_to_none() {
        assert_eq!(WebserverApiKey::default(), WebserverApiKey(None));
    }
}
