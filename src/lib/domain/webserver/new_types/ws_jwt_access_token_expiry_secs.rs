use serde_derive::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A validated and formatted Db connection retry initial delay in seconds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WebserverJwtAccessTokenExpirySecs(u64);

impl fmt::Display for WebserverJwtAccessTokenExpirySecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum WebserverJwtAccessTokenExpirySecsError {}

impl WebserverJwtAccessTokenExpirySecs {
    pub fn new(
        raw_jwt_access_token_expiry_secs: &u64,
    ) -> Result<Self, WebserverJwtAccessTokenExpirySecsError> {
        // Add validation logic here if needed
        Ok(WebserverJwtAccessTokenExpirySecs(
            *raw_jwt_access_token_expiry_secs,
        ))
    }
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl Default for WebserverJwtAccessTokenExpirySecs {
    fn default() -> Self {
        WebserverJwtAccessTokenExpirySecs(900)
    }
}
