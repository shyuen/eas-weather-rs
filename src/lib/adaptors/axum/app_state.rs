use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use std::sync::Arc;

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
pub struct AppState<CP, AP>
where
    CP: ConfigPort,
    AP: AlertPort,
{
    config_service: Arc<ConfigService<CP>>,
    alert_service: Arc<AlertService<AP>>,
}

impl<CP, AP> AppState<CP, AP>
where
    CP: ConfigPort,
    AP: AlertPort,
{
    /// Create a new AppState with the given services.
    pub fn new(config_service: ConfigService<CP>, alert_service: AlertService<AP>) -> Self {
        Self {
            config_service: Arc::new(config_service),
            alert_service: Arc::new(alert_service),
        }
    }

    /// Get a reference to the ConfigService.
    pub fn get_config_service(&self) -> Arc<ConfigService<CP>> {
        self.config_service.clone()
    }

    /// Get a reference to the AlertService.
    pub fn get_alert_service(&self) -> Arc<AlertService<AP>> {
        self.alert_service.clone()
    }
}
