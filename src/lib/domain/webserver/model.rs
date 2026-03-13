use crate::domain::config::model::ConfigWebserver;
use crate::domain::logging::port::LoggingRepo;
use crate::domain::webserver::sub_models::ws_api_key::WebserverApiKey;
use crate::domain::webserver::sub_models::ws_api_key::WebserverApiKeyError;
use crate::domain::webserver::sub_models::ws_base_path::WebserverBasePath;
use crate::domain::webserver::sub_models::ws_hostname::WebserverHostname;
use crate::domain::webserver::sub_models::ws_hostname::WebserverHostnameError;
use crate::domain::webserver::sub_models::ws_jwt_access_token_expiry_secs::WebserverJwtAccessTokenExpirySecs;
use crate::domain::webserver::sub_models::ws_jwt_key::WebserverJwtKey;
use crate::domain::webserver::sub_models::ws_jwt_key::WebserverJwtKeyError;
use crate::domain::webserver::sub_models::ws_port::WebserverPort;
use crate::domain::webserver::sub_models::ws_shutdown_timeout_secs::WebserverShutdownTimeoutSecs;

#[derive(Debug, Clone)]
pub struct Webserver {
    pub hostname: WebserverHostname,
    pub port: WebserverPort,
    pub base_path: WebserverBasePath,

    pub shutdown_timeout_secs: WebserverShutdownTimeoutSecs,

    pub api_key: WebserverApiKey,

    pub jwt_key: WebserverJwtKey,
    pub jwt_access_token_expiry_secs: WebserverJwtAccessTokenExpirySecs,
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

        Webserver {
            hostname,
            port,
            base_path,
            shutdown_timeout_secs,
            api_key,
            jwt_key,
            jwt_access_token_expiry_secs,
        }
    }

    pub fn validate_raw_config(&self, log_serv: &impl LoggingRepo, raw_ws_conf: &ConfigWebserver) {
        match &raw_ws_conf.hostname {
            Some(raw_hostname) => {
                if let Err(err) = WebserverHostname::new(raw_hostname) {
                    match &err {
                        WebserverHostnameError::EmptyHostname => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver hostname was empty, setting to `{}`",
                                    WebserverHostname::default()
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config webserver hostname was not specified, setting to `{}`",
                        WebserverHostname::default()
                    ),
                );
            }
        }

        match &raw_ws_conf.port {
            Some(raw_port) => {
                if let Err(err) = WebserverPort::new(raw_port) {
                    match &err {
                        _ => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver port of invalid value `{}`, setting to `{}`",
                                    raw_port,
                                    WebserverPort::default(),
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config webserver port was not specified, setting to `{}`",
                        WebserverPort::default()
                    ),
                );
            }
        }

        match &raw_ws_conf.base_path {
            Some(raw_base_path) => {
                if let Err(err) = WebserverBasePath::new(raw_base_path) {
                    match &err {
                        _ => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver base path of invalid value `{}`, setting to `{}`",
                                    raw_base_path,
                                    WebserverBasePath::default(),
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config webserver base path was not specified, setting to `{}`",
                        WebserverBasePath::default()
                    ),
                );
            }
        }

        match &raw_ws_conf.shutdown_timeout_secs {
            Some(raw_shutdown_timeout_secs) => {
                if let Err(err) = WebserverShutdownTimeoutSecs::new(raw_shutdown_timeout_secs) {
                    match &err {
                        _ => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver shutdown timeout secs of invalid value `{}`, setting to `{}`",
                                    raw_shutdown_timeout_secs,
                                    WebserverShutdownTimeoutSecs::default(),
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config webserver shutdown timeout secs was not specified, setting to `{}`",
                        WebserverShutdownTimeoutSecs::default()
                    ),
                );
            }
        }

        match &raw_ws_conf.api_key_file {
            Some(raw_api_key_file) => {
                if let Err(err) = WebserverApiKey::new(raw_api_key_file) {
                    match &err {
                        WebserverApiKeyError::BadFileLoad(e) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver load api key file error: `{}` - `{}`",
                                    raw_api_key_file, e,
                                ),
                            );
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver load api key will be set to `{}`",
                                    WebserverApiKey::default()
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!("config webserver api key file was not specified",),
                );
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config webserver api key will be set to `{}`",
                        WebserverApiKey::default()
                    ),
                );
            }
        }

        match &raw_ws_conf.jwt_key_file {
            Some(raw_jwt_key_file) => {
                if let Err(err) = WebserverJwtKey::new(raw_jwt_key_file) {
                    match &err {
                        WebserverJwtKeyError::BadFileLoad(e) => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver load jwt key file error: `{}` - `{}`",
                                    raw_jwt_key_file, e,
                                ),
                            );
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver jwt key will be set to `{}`",
                                    WebserverJwtKey::default()
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!("config webserver jwt key file was not specified",),
                );
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config webserver jwt key will be set to `{}`",
                        WebserverJwtKey::default()
                    ),
                );
            }
        }

        match &raw_ws_conf.jwt_access_token_expiry_secs {
            Some(raw_jwt_access_token_expiry_secs) => {
                if let Err(err) =
                    WebserverJwtAccessTokenExpirySecs::new(raw_jwt_access_token_expiry_secs)
                {
                    match &err {
                        _ => {
                            log_serv.warn(
                                module_path!(),
                                &format!(
                                    "config webserver jwt access token expiry secs of invalid value `{}`, setting to `{}`",
                                    raw_jwt_access_token_expiry_secs,
                                    WebserverJwtAccessTokenExpirySecs::default(),
                                ),
                            );
                        }
                    }
                }
            }
            None => {
                log_serv.warn(
                    module_path!(),
                    &format!(
                        "config webserver jwt access token expiry secs was not specified, setting to `{}`",
                        WebserverJwtAccessTokenExpirySecs::default()
                    ),
                );
            }
        }
    }
}
