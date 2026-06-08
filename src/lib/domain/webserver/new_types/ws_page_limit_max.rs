use serde_derive::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WebserverPageLimitMax(u64);

impl fmt::Display for WebserverPageLimitMax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum WebserverPageLimitMaxError {}

impl WebserverPageLimitMax {
    pub fn new(raw: &u64) -> Result<Self, WebserverPageLimitMaxError> {
        Ok(WebserverPageLimitMax(*raw))
    }
    pub fn default() -> Self {
        WebserverPageLimitMax(100)
    }
    pub fn get(&self) -> u64 {
        self.0
    }
}
