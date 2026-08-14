use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct DbMaxConnections(u32);

impl fmt::Display for DbMaxConnections {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum DbMaxConnectionsError {}

impl DbMaxConnections {
    pub fn new(raw_db_max_connections: &u32) -> Result<Self, DbMaxConnectionsError> {
        // Add validation logic here if needed
        Ok(DbMaxConnections(*raw_db_max_connections))
    }
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl Default for DbMaxConnections {
    fn default() -> Self {
        DbMaxConnections(151)
    }
}
