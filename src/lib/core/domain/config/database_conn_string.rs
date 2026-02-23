use std::fmt;
use std::fs;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DbConnectionString(String);

impl fmt::Display for DbConnectionString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum DbConnectionStringError {
    #[error("database connection string was empty")]
    EmptyConnectionString(String),
    #[error("database connection string could not be loaded from file")]
    BadFileLoad(String),
    #[error("database connection string file path was empty")]
    EmptyFilePath(String),
}

impl DbConnectionString {
    pub fn new(raw_conn_url_file: &str) -> Result<Self, DbConnectionStringError> {
        let trimmed_raw_file_path = raw_conn_url_file.trim().to_string();

        // Check if given file path was empty
        if trimmed_raw_file_path.is_empty() {
            return Err(DbConnectionStringError::EmptyFilePath("".to_string()));
        }

        // Read the string from the file
        let file_contents = match fs::read_to_string(trimmed_raw_file_path) {
            Ok(raw_conn_url) => raw_conn_url,
            Err(e) => {
                // Some issue loading the file
                return Err(DbConnectionStringError::BadFileLoad(e.to_string()));
            }
        };

        // Check if the connection string was empty
        let trimmed_file_contents = file_contents.trim();
        if trimmed_file_contents.is_empty() {
            return Err(DbConnectionStringError::EmptyConnectionString(
                "".to_string(),
            ));
        }

        // TODO - Other validation of the connection string as neccessary

        Ok(DbConnectionString(trimmed_file_contents.to_string()))
    }

    pub fn default() -> Self {
        DbConnectionString(
            format!("mysql://root@localhost:3306/{}", env!("CARGO_PKG_NAME")).to_string(),
        )
    }
}
