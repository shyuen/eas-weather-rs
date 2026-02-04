use std::fmt;
use std::fs;
use thiserror::Error;

#[derive(Debug)]
pub struct WebserverJwtKey(Option<String>);

impl fmt::Display for WebserverJwtKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(key) => write!(f, "{}", key.chars().map(|_| '*').collect::<String>()),
            None => write!(f, "<None>"),
        }
    }
}

#[derive(Error, Debug)]
pub enum WebserverJwtKeyError {
    #[error("database connection string could not be loaded from file")]
    BadFileLoad(String),
}

impl WebserverJwtKey {
    pub fn new(raw_jwt_key_file: &str) -> Result<Self, WebserverJwtKeyError> {
        let trimmed_raw_file_path = raw_jwt_key_file.trim().to_string();

        // Check if given file path was empty
        if trimmed_raw_file_path.is_empty() {
            return Ok(WebserverJwtKey(None));
        }

        // Read the string from the file
        let file_contents = match fs::read_to_string(trimmed_raw_file_path) {
            Ok(raw_api_key) => raw_api_key,
            Err(e) => {
                // Some issue loading the file
                return Err(WebserverJwtKeyError::BadFileLoad(e.to_string()));
            }
        };

        // Check if the connection string was empty
        let trimmed_file_contents = file_contents.trim();
        if trimmed_file_contents.is_empty() {
            return Ok(WebserverJwtKey(None));
        }

        // TODO - Other validation of the connection string as neccessary

        Ok(WebserverJwtKey(Some(trimmed_file_contents.to_string())))
    }

    pub fn default() -> Self {
        WebserverJwtKey(None)
    }
}
