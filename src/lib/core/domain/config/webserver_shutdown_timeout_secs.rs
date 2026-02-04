use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WebserverShutdownTimeoutSecs(u64);

impl fmt::Display for WebserverShutdownTimeoutSecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum WebserverShutdownTimeoutSecsError {}

impl WebserverShutdownTimeoutSecs {
    pub fn new(raw_shutdown_timeout_secs: &u64) -> Result<Self, WebserverShutdownTimeoutSecsError> {
        // Add validation logic here if needed
        Ok(WebserverShutdownTimeoutSecs(*raw_shutdown_timeout_secs))
    }
    pub fn default() -> Self {
        WebserverShutdownTimeoutSecs(300)
    }
    pub fn get(&self) -> u64 {
        self.0
    }
}
