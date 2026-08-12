use serde::{Deserialize, Serialize};

use crate::domain::config::issue::ConfigIssue;
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

/// Why the web server stopped, reported back to the caller so it can log the
/// shutdown appropriately without the adaptor owning the logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownReason {
    /// Received SIGINT (Ctrl+C).
    CtrlC,
    /// Received a terminate signal (SIGTERM on Unix).
    Terminate,
    /// The server stopped without a signal being received.
    Stopped,
}

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
            Some(raw_hostname) => WebserverHostname::new(raw_hostname)
                .unwrap_or_else(|_| WebserverHostname::default()),
            None => WebserverHostname::default(),
        };

        let port = match &conf.port {
            Some(raw_port) => WebserverPort::new(raw_port).unwrap_or_else(|_| {
                eprintln!("uncaught WebPortError");
                std::process::exit(1); // We use exit for planned exits instead of panics
            }),
            None => WebserverPort::default(),
        };

        let base_path = match &conf.base_path {
            Some(raw_base_path) => WebserverBasePath::new(raw_base_path).unwrap_or_else(|_| {
                eprintln!("uncaught WebserverBasePathError");
                std::process::exit(1); // We use exit for planned exits instead of panics
            }),
            None => WebserverBasePath::default(),
        };

        let shutdown_timeout_secs = match &conf.shutdown_timeout_secs {
            Some(raw_shutdown_timeout_secs) => {
                WebserverShutdownTimeoutSecs::new(raw_shutdown_timeout_secs).unwrap_or_else(|_| {
                    eprintln!("uncaught WebserverShutdownTimeoutSecsError");
                    std::process::exit(1); // We use exit for planned exits instead of panics
                })
            }
            None => WebserverShutdownTimeoutSecs::default(),
        };

        let api_key = match &conf.api_key_file {
            Some(raw_api_key_file) => WebserverApiKey::new(raw_api_key_file)
                .unwrap_or_else(|_| WebserverApiKey::default()),
            None => WebserverApiKey::default(),
        };

        let jwt_key = match &conf.jwt_key_file {
            Some(raw_jwt_key_file) => WebserverJwtKey::new(raw_jwt_key_file)
                .unwrap_or_else(|_| WebserverJwtKey::default()),
            None => WebserverJwtKey::default(),
        };

        let jwt_access_token_expiry_secs = match &conf.jwt_access_token_expiry_secs {
            Some(raw_jwt_access_token_expiry_secs) => {
                WebserverJwtAccessTokenExpirySecs::new(raw_jwt_access_token_expiry_secs)
                    .unwrap_or_else(|_| {
                        eprintln!("uncaught WebserverJwtAccessTokenExpirySecsError");
                        std::process::exit(1); // We use exit for planned exits instead of panics
                    })
            }
            None => WebserverJwtAccessTokenExpirySecs::default(),
        };

        let default_page_limit = match &conf.default_page_limit {
            Some(raw) => WebserverDefaultPageLimit::new(raw).unwrap_or_else(|_| {
                eprintln!("uncaught WebserverDefaultPageLimitError");
                std::process::exit(1);
            }),
            None => WebserverDefaultPageLimit::default(),
        };

        let page_limit_max = match &conf.page_limit_max {
            Some(raw) => WebserverPageLimitMax::new(raw).unwrap_or_else(|_| {
                eprintln!("uncaught WebserverPageLimitMaxError");
                std::process::exit(1);
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

    pub fn validate_raw_config(&self, raw_ws_conf: &ConfigWebserver) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        match &raw_ws_conf.hostname {
            Some(raw_hostname) => {
                if WebserverHostname::new(raw_hostname).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "webserver.hostname",
                        value: raw_hostname.clone(),
                        default: WebserverHostname::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.hostname",
                    default: WebserverHostname::default().to_string(),
                });
            }
        }

        match &raw_ws_conf.port {
            Some(raw_port) => {
                if WebserverPort::new(raw_port).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "webserver.port",
                        value: raw_port.to_string(),
                        default: WebserverPort::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.port",
                    default: WebserverPort::default().to_string(),
                });
            }
        }

        match &raw_ws_conf.base_path {
            Some(raw_base_path) => {
                if WebserverBasePath::new(raw_base_path).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "webserver.base_path",
                        value: raw_base_path.clone(),
                        default: WebserverBasePath::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.base_path",
                    default: WebserverBasePath::default().to_string(),
                });
            }
        }

        match &raw_ws_conf.shutdown_timeout_secs {
            Some(raw_shutdown_timeout_secs) => {
                if WebserverShutdownTimeoutSecs::new(raw_shutdown_timeout_secs).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "webserver.shutdown_timeout_secs",
                        value: raw_shutdown_timeout_secs.to_string(),
                        default: WebserverShutdownTimeoutSecs::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.shutdown_timeout_secs",
                    default: WebserverShutdownTimeoutSecs::default().to_string(),
                });
            }
        }

        match &raw_ws_conf.api_key_file {
            Some(raw_api_key_file) => {
                if let Err(err) = WebserverApiKey::new(raw_api_key_file) {
                    match &err {
                        WebserverApiKeyError::BadFileLoad(e) => {
                            issues.push(ConfigIssue::LoadFailed {
                                key: "webserver.api_key",
                                path: raw_api_key_file.clone(),
                                reason: e.to_string(),
                                default: WebserverApiKey::default().to_string(),
                            });
                        }
                    }
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.api_key",
                    default: WebserverApiKey::default().to_string(),
                });
            }
        }

        match &raw_ws_conf.jwt_key_file {
            Some(raw_jwt_key_file) => {
                if let Err(err) = WebserverJwtKey::new(raw_jwt_key_file) {
                    match &err {
                        WebserverJwtKeyError::BadFileLoad(e) => {
                            issues.push(ConfigIssue::LoadFailed {
                                key: "webserver.jwt_key",
                                path: raw_jwt_key_file.clone(),
                                reason: e.to_string(),
                                default: WebserverJwtKey::default().to_string(),
                            });
                        }
                    }
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.jwt_key",
                    default: WebserverJwtKey::default().to_string(),
                });
            }
        }

        match &raw_ws_conf.jwt_access_token_expiry_secs {
            Some(raw_jwt_access_token_expiry_secs) => {
                if WebserverJwtAccessTokenExpirySecs::new(raw_jwt_access_token_expiry_secs).is_err()
                {
                    issues.push(ConfigIssue::Invalid {
                        key: "webserver.jwt_access_token_expiry_secs",
                        value: raw_jwt_access_token_expiry_secs.to_string(),
                        default: WebserverJwtAccessTokenExpirySecs::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.jwt_access_token_expiry_secs",
                    default: WebserverJwtAccessTokenExpirySecs::default().to_string(),
                });
            }
        }

        match &raw_ws_conf.default_page_limit {
            Some(raw) => {
                if WebserverDefaultPageLimit::new(raw).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "webserver.default_page_limit",
                        value: raw.to_string(),
                        default: WebserverDefaultPageLimit::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.default_page_limit",
                    default: WebserverDefaultPageLimit::default().to_string(),
                });
            }
        }

        match &raw_ws_conf.page_limit_max {
            Some(raw) => {
                if WebserverPageLimitMax::new(raw).is_err() {
                    issues.push(ConfigIssue::Invalid {
                        key: "webserver.page_limit_max",
                        value: raw.to_string(),
                        default: WebserverPageLimitMax::default().to_string(),
                    });
                }
            }
            None => {
                issues.push(ConfigIssue::NotSpecified {
                    key: "webserver.page_limit_max",
                    default: WebserverPageLimitMax::default().to_string(),
                });
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn raw(
        hostname: Option<&str>,
        port: Option<u16>,
        base_path: Option<&str>,
        shutdown_timeout_secs: Option<u64>,
        api_key_file: Option<&str>,
        jwt_key_file: Option<&str>,
        jwt_access_token_expiry_secs: Option<u64>,
        default_page_limit: Option<u64>,
        page_limit_max: Option<u64>,
    ) -> ConfigWebserver {
        ConfigWebserver {
            hostname: hostname.map(str::to_string),
            port,
            base_path: base_path.map(str::to_string),
            shutdown_timeout_secs,
            api_key_file: api_key_file.map(str::to_string),
            jwt_key_file: jwt_key_file.map(str::to_string),
            jwt_access_token_expiry_secs,
            default_page_limit,
            page_limit_max,
        }
    }

    fn empty() -> ConfigWebserver {
        raw(None, None, None, None, None, None, None, None, None)
    }

    fn loadable_secret_files() -> (tempfile::NamedTempFile, tempfile::NamedTempFile) {
        use std::io::Write;
        let mut api = tempfile::NamedTempFile::new().unwrap();
        writeln!(api, "api-secret").unwrap();
        let mut jwt = tempfile::NamedTempFile::new().unwrap();
        writeln!(jwt, "jwt-secret").unwrap();
        (api, jwt)
    }

    #[test]
    fn validate_accepts_valid_values() {
        let (api_key, jwt_key) = loadable_secret_files();
        let issues = Webserver::new(&empty()).validate_raw_config(&raw(
            Some("localhost"),
            Some(8080),
            Some("/api"),
            Some(5),
            Some(api_key.path().to_str().unwrap()),
            Some(jwt_key.path().to_str().unwrap()),
            Some(3600),
            Some(10),
            Some(50),
        ));
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_flags_empty_hostname() {
        let (api_key, jwt_key) = loadable_secret_files();
        let issues = Webserver::new(&empty()).validate_raw_config(&raw(
            Some("  "),
            Some(8080),
            Some("/api"),
            Some(5),
            Some(api_key.path().to_str().unwrap()),
            Some(jwt_key.path().to_str().unwrap()),
            Some(3600),
            Some(10),
            Some(50),
        ));
        assert_eq!(
            issues,
            vec![ConfigIssue::Invalid {
                key: "webserver.hostname",
                value: "  ".into(),
                default: "\"localhost\"".into(),
            }]
        );
    }

    #[test]
    fn validate_flags_missing_settings() {
        let issues = Webserver::new(&empty()).validate_raw_config(&empty());
        let keys: Vec<_> = issues
            .iter()
            .map(|issue| match issue {
                ConfigIssue::NotSpecified { key, .. } => *key,
                _ => panic!("expected NotSpecified, got {:?}", issue),
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                "webserver.hostname",
                "webserver.port",
                "webserver.base_path",
                "webserver.shutdown_timeout_secs",
                "webserver.api_key",
                "webserver.jwt_key",
                "webserver.jwt_access_token_expiry_secs",
                "webserver.default_page_limit",
                "webserver.page_limit_max",
            ]
        );
    }

    #[test]
    fn validate_flags_unloadable_api_key_file() {
        let (_, jwt_key) = loadable_secret_files();
        let issues = Webserver::new(&empty()).validate_raw_config(&raw(
            Some("localhost"),
            Some(8080),
            Some("/api"),
            Some(5),
            Some("/nonexistent/api.key"),
            Some(jwt_key.path().to_str().unwrap()),
            Some(3600),
            Some(10),
            Some(50),
        ));
        let load_failed: Vec<_> = issues
            .iter()
            .filter_map(|issue| match issue {
                ConfigIssue::LoadFailed { key, .. } => Some(*key),
                _ => None,
            })
            .collect();
        assert_eq!(load_failed, vec!["webserver.api_key"]);
    }

    #[test]
    fn validate_flags_unloadable_jwt_key_file() {
        let (api_key, _) = loadable_secret_files();
        let issues = Webserver::new(&empty()).validate_raw_config(&raw(
            Some("localhost"),
            Some(8080),
            Some("/api"),
            Some(5),
            Some(api_key.path().to_str().unwrap()),
            Some("/nonexistent/jwt.key"),
            Some(3600),
            Some(10),
            Some(50),
        ));
        let load_failed: Vec<_> = issues
            .iter()
            .filter_map(|issue| match issue {
                ConfigIssue::LoadFailed { key, .. } => Some(*key),
                _ => None,
            })
            .collect();
        assert_eq!(load_failed, vec!["webserver.jwt_key"]);
    }
}
