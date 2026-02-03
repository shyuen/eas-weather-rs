use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WebserverPort(u16);

impl fmt::Display for WebserverPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum WebserverPortError {}

impl WebserverPort {
    pub fn new(raw_ws_port: &u16) -> Result<Self, WebserverPortError> {
        // Add validation logic here if needed
        Ok(WebserverPort(*raw_ws_port))
    }
    pub fn default() -> Self {
        WebserverPort(8080)
    }
    pub fn get(&self) -> u16 {
        self.0
    }
}
