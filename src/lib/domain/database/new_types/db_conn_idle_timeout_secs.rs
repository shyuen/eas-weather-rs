use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct DbConnIdleTimeoutSecs(u32);

impl fmt::Display for DbConnIdleTimeoutSecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum DbConnIdleTimeoutSecsError {}

impl DbConnIdleTimeoutSecs {
    pub fn new(raw_db_conn_idle_timeout_secs: &u32) -> Result<Self, DbConnIdleTimeoutSecsError> {
        // Add validation logic here if needed
        Ok(DbConnIdleTimeoutSecs(*raw_db_conn_idle_timeout_secs))
    }
    pub fn default() -> Self {
        DbConnIdleTimeoutSecs(300)
    }
    pub fn get(&self) -> u32 {
        self.0
    }
}
