use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct DbConnAcquireTimeoutSecs(u16);

impl fmt::Display for DbConnAcquireTimeoutSecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum DbConnAcquireTimeoutSecsError {}

impl DbConnAcquireTimeoutSecs {
    pub fn new(
        raw_db_conn_aquire_timeout_secs: &u16,
    ) -> Result<Self, DbConnAcquireTimeoutSecsError> {
        // Add validation logic here if needed
        Ok(DbConnAcquireTimeoutSecs(*raw_db_conn_aquire_timeout_secs))
    }
    pub fn get(&self) -> u16 {
        self.0
    }
}

impl Default for DbConnAcquireTimeoutSecs {
    fn default() -> Self {
        DbConnAcquireTimeoutSecs(5)
    }
}
