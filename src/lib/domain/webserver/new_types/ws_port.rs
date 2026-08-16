use serde_derive::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WebserverTcpPort(u16);

impl fmt::Display for WebserverTcpPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum WebserverTcpPortError {}

impl WebserverTcpPort {
    pub fn new(raw_ws_port: &u16) -> Result<Self, WebserverTcpPortError> {
        // Add validation logic here if needed
        Ok(WebserverTcpPort(*raw_ws_port))
    }
    pub fn get(&self) -> u16 {
        self.0
    }
}

impl Default for WebserverTcpPort {
    fn default() -> Self {
        WebserverTcpPort(8080)
    }
}
