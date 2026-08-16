use crate::domain::alert::port::AlertPort;
use crate::domain::alert::service::AlertService;
use crate::domain::config::port::MetaPort;
use crate::domain::database::port::DatabasePort;
use std::sync::Arc;

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
pub struct AppState<MR, DR>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    meta_port: Arc<MR>,
    alert_service: Arc<AlertService<DR>>,
}

impl<MR, DR> AppState<MR, DR>
where
    MR: MetaPort,
    DR: DatabasePort + AlertPort,
{
    /// Create a new AppState with the given services.
    pub fn new(meta_port: MR, alert_service: AlertService<DR>) -> Self {
        Self {
            meta_port: Arc::new(meta_port),
            alert_service: Arc::new(alert_service),
        }
    }

    /// Get a reference to the MetaService.
    pub fn get_meta_port(&self) -> Arc<MR> {
        self.meta_port.clone()
    }

    /// Get a reference to the AlertService.
    pub fn get_alert_service(&self) -> Arc<AlertService<DR>> {
        self.alert_service.clone()
    }
}
