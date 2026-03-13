use std::fmt;
use thiserror::Error;

/// A validated and formatted MySQL connection max retries.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DbConnMaxRetries(u8);

impl fmt::Display for DbConnMaxRetries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum DbConnMaxRetriesError {}

impl DbConnMaxRetries {
    pub fn new(raw_db_conn_max_retries: &u8) -> Result<Self, DbConnMaxRetriesError> {
        // Add validation logic here if needed
        Ok(DbConnMaxRetries(*raw_db_conn_max_retries))
    }
    pub fn default() -> Self {
        DbConnMaxRetries(5)
    }
    pub fn get(&self) -> u8 {
        self.0
    }
}
