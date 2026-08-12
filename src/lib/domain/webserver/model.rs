use serde::{Deserialize, Serialize};

use crate::domain::config::model::ConfigWebserver;
use crate::domain::utils::helpers::serialize_with_display;
use crate::domain::webserver::new_types::ws_api_key::WebserverApiKey;
use crate::domain::webserver::new_types::ws_api_key::WebserverApiKeyError;
use crate::domain::webserver::new_types::ws_base_path::WebserverBasePath;
use crate::domain::webserver::new_types::ws_default_page_limit::WebserverDefaultPageLimit;
use crate::domain::webserver::new_types::ws_hostname::WebserverHostname;
use crate::domain::webserver::new_types::ws_jwt_access_token_expiry_secs::WebserverJwtAccessTokenExpirySecs;
use crate::domain::webserver::new_types::ws_jwt_key::WebserverJwtKey;
use crate::domain::webserver::new_types::ws_jwt_key::WebserverJwtKeyError;
use crate::domain::webserver::new_types::ws_page_limit_max::WebserverPageLimitMax;
use crate::domain::webserver::new_types::ws_port::WebserverPort;
use crate::domain::webserver::new_types::ws_shutdown_timeout_secs::WebserverShutdownTimeoutSecs;
use crate::warn_config_invalid;
use crate::warn_config_load_failed;
use crate::warn_config_not_specified;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webserver {
    pub hostname: WebserverHostname,
    pub port: WebserverPort,
    pub base_path: WebserverBasePath,

    pub shutdown_timeout_secs: WebserverShutdownTimeoutSecs,

    #[serde(serialize_with = "serialize_with_display")]
    pub api_key: WebserverApiKey,

    #[serde(serialize_with = "serialize_with_display")]
    pub jwt_key: WebserverJwtKey,
    pub jwt_access_token_expiry_secs: WebserverJwtAccessTokenExpirySecs,

    pub default_page_limit: WebserverDefaultPageLimit,
    pub page_limit_max: WebserverPageLimitMax,
}

impl Webserver {
    // Creates a new instance of Webserver configuration.
    pub fn new(conf: &ConfigWebserver) -> Self {
        let hostname = match &conf.hostname {
            Some(raw_hostname) => {
                WebserverHostname::new(raw_hostname).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    _ => WebserverHostname::default(),
                })
            }
            None => WebserverHostname::default(),
        };

        let port = match &conf.port {
            Some(raw_port) => {
                WebserverPort::new(raw_port).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    _ => {
                        eprintln!("uncaught WebPortError");
                        std::process::exit(1); // We use exit for planned exits instead of panics
                    }
                })
            }
            None => WebserverPort::default(),
        };

        let base_path = match &conf.base_path {
            Some(raw_base_path) => {
                WebserverBasePath::new(raw_base_path).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    _ => {
                        eprintln!("uncaught WebserverBasePathError");
                        std::process::exit(1); // We use exit for planned exits instead of panics
                    }
                })
            }
            None => WebserverBasePath::default(),
        };

        let shutdown_timeout_secs = match &conf.shutdown_timeout_secs {
            Some(raw_shutdown_timeout_secs) => {
                WebserverShutdownTimeoutSecs::new(raw_shutdown_timeout_secs).unwrap_or_else(|err| {
                    match &err {
                        // Set to default the default option on errors
                        // We don't handle logging here as the logger is not yet initialized
                        _ => {
                            eprintln!("uncaught WebserverShutdownTimeoutSecsError");
                            std::process::exit(1); // We use exit for planned exits instead of panics
                        }
                    }
                })
            }
            None => WebserverShutdownTimeoutSecs::default(),
        };

        let api_key = match &conf.api_key_file {
            Some(raw_api_key_file) => {
                WebserverApiKey::new(raw_api_key_file).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    _ => WebserverApiKey::default(),
                })
            }
            None => WebserverApiKey::default(),
        };

        let jwt_key = match &conf.jwt_key_file {
            Some(raw_jwt_key_file) => {
                WebserverJwtKey::new(raw_jwt_key_file).unwrap_or_else(|err| match &err {
                    // Set to default the default option on errors
                    // We don't handle logging here as the logger is not yet initialized
                    _ => WebserverJwtKey::default(),
                })
            }
            None => WebserverJwtKey::default(),
        };

        let jwt_access_token_expiry_secs = match &conf.jwt_access_token_expiry_secs {
            Some(raw_jwt_access_token_expiry_secs) => {
                WebserverJwtAccessTokenExpirySecs::new(raw_jwt_access_token_expiry_secs)
                    .unwrap_or_else(|err| match &err {
                        // Set to default the default option on errors
                        // We don't handle logging here as the logger is not yet initialized
                        _ => {
                            eprintln!("uncaught WebserverJwtAccessTokenExpirySecsError");
                            std::process::exit(1); // We use exit for planned exits instead of panics
                        }
                    })
            }
            None => WebserverJwtAccessTokenExpirySecs::default(),
        };

        let default_page_limit = match &conf.default_page_limit {
            Some(raw) => WebserverDefaultPageLimit::new(raw).unwrap_or_else(|err| match &err {
                _ => {
                    eprintln!("uncaught WebserverDefaultPageLimitError");
                    std::process::exit(1);
                }
            }),
            None => WebserverDefaultPageLimit::default(),
        };

        let page_limit_max = match &conf.page_limit_max {
            Some(raw) => WebserverPageLimitMax::new(raw).unwrap_or_else(|err| match &err {
                _ => {
                    eprintln!("uncaught WebserverPageLimitMaxError");
                    std::process::exit(1);
                }
            }),
            None => WebserverPageLimitMax::default(),
        };

        Webserver {
            hostname,
            port,
            base_path,
            shutdown_timeout_secs,
            api_key,
            jwt_key,
            jwt_access_token_expiry_secs,
            default_page_limit,
            page_limit_max,
        }
    }

    pub fn validate_raw_config(&self, raw_ws_conf: &ConfigWebserver) {
        match &raw_ws_conf.hostname {
            Some(raw_hostname) => {
                if WebserverHostname::new(raw_hostname).is_err() {
                    warn_config_invalid!(
                        "webserver.hostname",
                        raw_hostname,
                        WebserverHostname::default(),
                    );
                }
            }
            None => {
                warn_config_not_specified!("webserver.hostname", WebserverHostname::default());
            }
        }

        match &raw_ws_conf.port {
            Some(raw_port) => {
                if WebserverPort::new(raw_port).is_err() {
                    warn_config_invalid!(
                        "webserver.port",
                        &raw_port.to_string(),
                        WebserverPort::default(),
                    );
                }
            }
            None => warn_config_not_specified!("webserver.port", WebserverPort::default()),
        }

        match &raw_ws_conf.base_path {
            Some(raw_base_path) => {
                if WebserverBasePath::new(raw_base_path).is_err() {
                    warn_config_invalid!(
                        "webserver.base_path",
                        raw_base_path,
                        WebserverBasePath::default(),
                    );
                }
            }
            None => warn_config_not_specified!("webserver.base_path", WebserverBasePath::default()),
        }

        match &raw_ws_conf.shutdown_timeout_secs {
            Some(raw_shutdown_timeout_secs) => {
                if WebserverShutdownTimeoutSecs::new(raw_shutdown_timeout_secs).is_err() {
                    warn_config_invalid!(
                        "webserver.shutdown_timeout_secs",
                        &raw_shutdown_timeout_secs.to_string(),
                        WebserverShutdownTimeoutSecs::default(),
                    );
                }
            }
            None => warn_config_not_specified!(
                "webserver.shutdown_timeout_secs",
                WebserverShutdownTimeoutSecs::default(),
            ),
        }

        match &raw_ws_conf.api_key_file {
            Some(raw_api_key_file) => {
                if let Err(err) = WebserverApiKey::new(raw_api_key_file) {
                    match &err {
                        WebserverApiKeyError::BadFileLoad(e) => {
                            warn_config_load_failed!(
                                "webserver.api_key",
                                raw_api_key_file,
                                &e.to_string(),
                                WebserverApiKey::default(),
                            );
                        }
                    }
                }
            }
            None => {
                warn_config_not_specified!("webserver.api_key", WebserverApiKey::default());
            }
        }

        match &raw_ws_conf.jwt_key_file {
            Some(raw_jwt_key_file) => {
                if let Err(err) = WebserverJwtKey::new(raw_jwt_key_file) {
                    match &err {
                        WebserverJwtKeyError::BadFileLoad(e) => {
                            warn_config_load_failed!(
                                "webserver.jwt_key",
                                raw_jwt_key_file,
                                &e.to_string(),
                                WebserverJwtKey::default(),
                            );
                        }
                    }
                }
            }
            None => warn_config_not_specified!("webserver.jwt_key", WebserverJwtKey::default()),
        }

        match &raw_ws_conf.jwt_access_token_expiry_secs {
            Some(raw_jwt_access_token_expiry_secs) => {
                if WebserverJwtAccessTokenExpirySecs::new(raw_jwt_access_token_expiry_secs).is_err()
                {
                    warn_config_invalid!(
                        "webserver.jwt_access_token_expiry_secs",
                        &raw_jwt_access_token_expiry_secs.to_string(),
                        WebserverJwtAccessTokenExpirySecs::default(),
                    );
                }
            }
            None => warn_config_not_specified!(
                "webserver.jwt_access_token_expiry_secs",
                WebserverJwtAccessTokenExpirySecs::default(),
            ),
        }

        match &raw_ws_conf.default_page_limit {
            Some(raw) => {
                if WebserverDefaultPageLimit::new(raw).is_err() {
                    warn_config_invalid!(
                        "webserver.default_page_limit",
                        &raw.to_string(),
                        WebserverDefaultPageLimit::default(),
                    );
                }
            }
            None => warn_config_not_specified!(
                "webserver.default_page_limit",
                WebserverDefaultPageLimit::default(),
            ),
        }

        match &raw_ws_conf.page_limit_max {
            Some(raw) => {
                if WebserverPageLimitMax::new(raw).is_err() {
                    warn_config_invalid!(
                        "webserver.page_limit_max",
                        &raw.to_string(),
                        WebserverPageLimitMax::default(),
                    );
                }
            }
            None => warn_config_not_specified!(
                "webserver.page_limit_max",
                WebserverPageLimitMax::default(),
            ),
        }
    }
}
