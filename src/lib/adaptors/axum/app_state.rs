use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
use std::sync::Arc;

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
pub struct AppState<C, DR>
where
    C: ConfigPort,
    DR: DatabasePort + AlertPort,
{
    config_service: Arc<ConfigService<C>>,
    alert_service: Arc<AlertService<DR>>,
}

impl<C, DR> AppState<C, DR>
where
    C: ConfigPort,
    DR: DatabasePort + AlertPort,
{
    /// Create a new AppState with the given services.
    pub fn new(config_service: ConfigService<C>, alert_service: AlertService<DR>) -> Self {
        Self {
            config_service: Arc::new(config_service),
            alert_service: Arc::new(alert_service),
        }
    }

    /// Get a reference to the ConfigService.
    pub fn get_config_service(&self) -> Arc<ConfigService<C>> {
        self.config_service.clone()
    }

    /// Get a reference to the AlertService.
    pub fn get_alert_service(&self) -> Arc<AlertService<DR>> {
        self.alert_service.clone()
    }
}
