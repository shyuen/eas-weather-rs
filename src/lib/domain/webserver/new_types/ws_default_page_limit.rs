use serde_derive::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WebserverDefaultPageLimit(u64);

impl fmt::Display for WebserverDefaultPageLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum WebserverDefaultPageLimitError {}

impl WebserverDefaultPageLimit {
    pub fn new(raw: &u64) -> Result<Self, WebserverDefaultPageLimitError> {
        Ok(WebserverDefaultPageLimit(*raw))
    }
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl Default for WebserverDefaultPageLimit {
    fn default() -> Self {
        WebserverDefaultPageLimit(100)
    }
}
