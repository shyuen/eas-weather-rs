use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DbConnRetryInitDelaySecs(u64);

#[derive(Error, Debug)]
pub enum DbConnRetryInitDelaySecsError {}

impl DbConnRetryInitDelaySecs {
    pub fn new(
        raw_db_conn_retry_init_delay_secs: &u64,
    ) -> Result<Self, DbConnRetryInitDelaySecsError> {
        // Add validation logic here if needed
        Ok(DbConnRetryInitDelaySecs(*raw_db_conn_retry_init_delay_secs))
    }
    pub fn default() -> Self {
        DbConnRetryInitDelaySecs(1)
    }
    pub fn get(&self) -> u64 {
        self.0
    }
}
