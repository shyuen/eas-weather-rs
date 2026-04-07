use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct DbConnMaxLifetimeSecs(u32);

impl fmt::Display for DbConnMaxLifetimeSecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum DbConnMaxLifetimeSecsError {}

impl DbConnMaxLifetimeSecs {
    pub fn new(raw_db_conn_max_lifetime_secs: &u32) -> Result<Self, DbConnMaxLifetimeSecsError> {
        // Add validation logic here if needed
        Ok(DbConnMaxLifetimeSecs(*raw_db_conn_max_lifetime_secs))
    }
    pub fn default() -> Self {
        DbConnMaxLifetimeSecs(1800)
    }
    pub fn get(&self) -> u32 {
        self.0
    }
}
