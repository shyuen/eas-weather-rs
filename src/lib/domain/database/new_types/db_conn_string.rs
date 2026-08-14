use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConnectionString(String);

impl fmt::Display for DbConnectionString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = self.0.to_string();
        let db_type_rest: Vec<&str> = string.split("://").collect();
        let cred_rest: Vec<&str> = db_type_rest[1].split('@').collect();

        // Create string that replaces all characters except `:` with a `*`
        let cred = cred_rest[0]
            .chars()
            .map(|c| if c == ':' { ':' } else { '*' })
            .collect::<String>();

        let conn_string = format!("{}://{}@{}", db_type_rest[0], cred, cred_rest[1]);

        write!(f, "{}", conn_string)
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

    pub fn get(&self) -> &str {
        &self.0
    }
}

impl Default for DbConnectionString {
    fn default() -> Self {
        DbConnectionString(
            format!("mysql://root@localhost:3306/{}", env!("CARGO_PKG_NAME")).to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn loads_connection_string_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "mysql://user:pass@localhost/eas_weather").unwrap();
        let conn = DbConnectionString::new(file.path().to_str().unwrap()).unwrap();
        assert_eq!(conn.get(), "mysql://user:pass@localhost/eas_weather");
    }

    #[test]
    fn trims_file_contents() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "  mysql://root@localhost/eas_weather").unwrap();
        let conn = DbConnectionString::new(file.path().to_str().unwrap()).unwrap();
        assert_eq!(conn.get(), "mysql://root@localhost/eas_weather");
    }

    #[test]
    fn rejects_empty_file_path() {
        assert!(matches!(
            DbConnectionString::new("").err().unwrap(),
            DbConnectionStringError::EmptyFilePath(_)
        ));
        assert!(matches!(
            DbConnectionString::new("   ").err().unwrap(),
            DbConnectionStringError::EmptyFilePath(_)
        ));
    }

    #[test]
    fn rejects_missing_file() {
        assert!(matches!(
            DbConnectionString::new("/nonexistent/conn.txt")
                .err()
                .unwrap(),
            DbConnectionStringError::BadFileLoad(_)
        ));
    }

    #[test]
    fn rejects_empty_connection_string() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "   ").unwrap();
        assert!(matches!(
            DbConnectionString::new(file.path().to_str().unwrap())
                .err()
                .unwrap(),
            DbConnectionStringError::EmptyConnectionString(_)
        ));
    }

    #[test]
    fn display_masks_credentials() {
        let db = DbConnectionString("mysql://user:pass@localhost/eas_weather".to_string());
        let shown = db.to_string();
        // Everything before the '@' (user:pass) must be masked; host is preserved.
        let cred_part = shown.split('@').next().unwrap();
        assert!(!cred_part.contains("user"), "username leaked: {cred_part}");
        assert!(!cred_part.contains("pass"), "password leaked: {cred_part}");
        assert!(
            shown.contains("localhost"),
            "host should be preserved: {shown}"
        );
    }

    #[test]
    fn default_builds_packaged_mysql_url() {
        let conn = DbConnectionString::default();
        assert!(conn.get().starts_with("mysql://"));
        assert!(conn.get().contains(env!("CARGO_PKG_NAME")));
    }
}
