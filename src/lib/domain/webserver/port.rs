use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
use crate::domain::logging::adaptor_config::AdaptorConfigRepr;
use crate::domain::webserver::model::{ShutdownReason, Webserver};
use thiserror::Error;

/// Errors that can occur while starting the web server.
#[derive(Debug, Error)]
pub enum WebserverStartError {
    /// The listener or serve loop returned an I/O error.
    #[error("webserver failed to start: {0}")]
    Io(std::io::Error),
}

impl WebserverStartError {
    /// Stable machine-readable code for this error, usable as a log key.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Io(err) => match err.kind() {
                std::io::ErrorKind::AddrInUse => "webserver_start_addr_in_use",
                std::io::ErrorKind::AddrNotAvailable => "webserver_start_addr_not_available",
                std::io::ErrorKind::PermissionDenied => "webserver_start_permission_denied",
                _ => "webserver_start_io_error",
            },
        }
    }
}

pub trait WebserverPort: AdaptorConfigRepr + Clone + Send + Sync + 'static {
    /// Create a new instance of the webserver repository with the given configuration
    fn new(conf_webserv: &Webserver) -> Self;

    /// Start the web server. The returned `ShutdownReason` indicates why the
    /// server stopped so the caller (service) can log it; the adaptor performs
    /// the shutdown but does not own the logging.
    fn start_server<CP, AP, DP>(
        &self,
        alert_service: &AlertService<AP>,
        config_service: &ConfigService<CP>,
        db_port: &DP,
    ) -> impl std::future::Future<Output = Result<ShutdownReason, WebserverStartError>> + Send
    where
        CP: ConfigPort,
        AP: AlertPort,
        DP: DatabasePort;
}

#[cfg(test)]
mod code_tests {
    use super::*;
    use std::io;

    #[test]
    fn webserver_start_error_codes() {
        assert_eq!(
            WebserverStartError::Io(io::Error::from(io::ErrorKind::AddrInUse)).code(),
            "webserver_start_addr_in_use"
        );
        assert_eq!(
            WebserverStartError::Io(io::Error::from(io::ErrorKind::AddrNotAvailable)).code(),
            "webserver_start_addr_not_available"
        );
        assert_eq!(
            WebserverStartError::Io(io::Error::from(io::ErrorKind::PermissionDenied)).code(),
            "webserver_start_permission_denied"
        );
        assert_eq!(
            WebserverStartError::Io(io::Error::other("boom")).code(),
            "webserver_start_io_error"
        );
    }
}
