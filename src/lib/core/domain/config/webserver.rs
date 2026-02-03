use crate::core::domain::config::webserver_api_key::WebserverApiKey;
use crate::core::domain::config::webserver_base_path::WebserverBasePath;
use crate::core::domain::config::webserver_hostname::WebserverHostname;
use crate::core::domain::config::webserver_jwt_access_token_expiry_secs::WebserverJwtAccessTokenExpirySecs;
use crate::core::domain::config::webserver_jwt_key::WebserverJwtKey;
use crate::core::domain::config::webserver_port::WebserverPort;
use crate::core::domain::config::webserver_shutdown_timeout_secs::WebserverShutdownTimeoutSecs;

#[derive(Debug)]
pub struct Webserver {
    pub hostname: WebserverHostname,
    pub port: WebserverPort,
    pub base_path: WebserverBasePath,

    pub shutdown_timeout_secs: WebserverShutdownTimeoutSecs,

    pub api_key_file: WebserverApiKey,

    pub jwt_key_file: WebserverJwtKey,
    pub jwt_access_token_expiry_secs: WebserverJwtAccessTokenExpirySecs,
}
