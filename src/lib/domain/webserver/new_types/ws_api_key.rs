use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WebserverApiKey(Option<String>);

impl fmt::Display for WebserverApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(key) => write!(f, "{}", key.chars().map(|_| '*').collect::<String>()),
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

    pub fn default() -> Self {
        WebserverApiKey(None)
    }

    pub fn get(&self) -> &Option<String> {
        &self.0
    }
}
