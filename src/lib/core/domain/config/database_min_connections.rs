use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DbMinConnections(u32);

impl fmt::Display for DbMinConnections {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum DbMinConnectionsError {}

impl DbMinConnections {
    pub fn new(raw_db_max_connections: &u32) -> Result<Self, DbMinConnectionsError> {
        // Add validation logic here if needed
        Ok(DbMinConnections(*raw_db_max_connections))
    }
    pub fn default() -> Self {
        DbMinConnections(1)
    }
    pub fn get(&self) -> u32 {
        self.0
    }
}
