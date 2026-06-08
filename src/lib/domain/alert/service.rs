use crate::domain::alert::port::{AlertPort, GetDailyAlertsError, GetDailyAlertsResponse};
use crate::domain::database::port::DatabasePort;
use crate::domain::database::service::DatabaseService;
use tracing::info;

/// The AlertService struct provides methods for managing alerts.
/// Contains ports to interact with other services
#[derive(Debug, Clone)]
pub struct AlertService<D>
where
    D: AlertPort + DatabasePort,
{
    db_port: D,
}

impl<D> AlertService<D>
where
    D: AlertPort + DatabasePort,
{
    /// Creates a new instance of AlertService.
    pub fn new(db_serv: DatabaseService<D>) -> Self {
        let db_port = db_serv.get_port();

        // Log the initialization of the AlertService
        info!("Initializing AlertService");

        Self {
            db_port: db_port.clone(),
        }
    }

    /// Get the Database repository
    pub fn get_db_port(&self) -> &D {
        &self.db_port
    }

    /// Retrieve the latest alerts sent within the last 24 hours from the database.
    pub async fn get_daily_alerts(&self) -> Result<GetDailyAlertsResponse, GetDailyAlertsError> {
        self.db_port.get_daily_alerts_data().await
    }
}
