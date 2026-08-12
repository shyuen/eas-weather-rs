use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct DbConnRetryInitDelaySecs(u16);

impl fmt::Display for DbConnRetryInitDelaySecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum DbConnRetryInitDelaySecsError {}

impl DbConnRetryInitDelaySecs {
    pub fn new(
        raw_db_conn_retry_init_delay_secs: &u16,
    ) -> Result<Self, DbConnRetryInitDelaySecsError> {
        // Add validation logic here if needed
        Ok(DbConnRetryInitDelaySecs(*raw_db_conn_retry_init_delay_secs))
    }
    pub fn get(&self) -> u16 {
        self.0
    }
}

impl Default for DbConnRetryInitDelaySecs {
    fn default() -> Self {
        DbConnRetryInitDelaySecs(1)
    }
}
