use crate::domain::alert::port::{
    AlertPort, GetDailyAlertsError, GetDailyAlertsResponse, GetLatestAlertsError,
    GetLatestAlertsResponse,
};
use crate::domain::database::port::DatabasePort;
use crate::domain::database::service::DatabaseService;
use tracing::{debug, error, info};

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
    pub async fn get_daily_alerts(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<GetDailyAlertsResponse, GetDailyAlertsError> {
        debug!("get_daily_alerts(limit={}, offset={})", limit, offset);
        match self.db_port.get_daily_alerts_data(limit, offset).await {
            Ok(resp) => {
                info!(
                    "get_daily_alerts returned {} of {} alerts",
                    resp.alerts.len(),
                    resp.total
                );
                Ok(resp)
            }
            Err(err) => {
                error!("get_daily_alerts failed: {}", err);
                Err(err)
            }
        }
    }

    /// Retrieve the latest alerts from the database.
    pub async fn get_latest_alerts(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<GetLatestAlertsResponse, GetLatestAlertsError> {
        debug!("get_latest_alerts(limit={}, offset={})", limit, offset);
        match self.db_port.get_latest_alerts_data(limit, offset).await {
            Ok(resp) => {
                info!(
                    "get_latest_alerts returned {} of {} alerts",
                    resp.alerts.len(),
                    resp.total
                );
                Ok(resp)
            }
            Err(err) => {
                error!("get_latest_alerts failed: {}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::database::service::DatabaseService;
    use crate::test_support::{FailingDb, MockConfig, MockDb};

    /// Build an `AlertService<D>` over the supplied `DatabasePort + AlertPort` double.
    fn build_service<D>() -> AlertService<D>
    where
        D: AlertPort + DatabasePort,
    {
        let conf_service = crate::domain::config::service::ConfigService {
            port: MockConfig::new(),
        };
        let db_service = DatabaseService::<D>::new(&conf_service);
        AlertService::new(db_service)
    }

    #[tokio::test]
    async fn get_latest_alerts_returns_port_data() {
        let service = build_service::<MockDb>();
        let resp = service.get_latest_alerts(10, 5).await.unwrap();
        assert_eq!(resp.total, 42);
        assert!(resp.alerts.is_empty());
    }

    #[tokio::test]
    async fn get_latest_alerts_propagates_error() {
        let service = build_service::<FailingDb>();
        let result = service.get_latest_alerts(10, 5).await;
        assert!(matches!(
            result,
            Err(GetLatestAlertsError::DatabaseError(msg)) if msg == "test error"
        ));
    }

    #[tokio::test]
    async fn get_daily_alerts_returns_port_data() {
        let service = build_service::<MockDb>();
        let resp = service.get_daily_alerts(10, 5).await.unwrap();
        assert_eq!(resp.total, 42);
        assert!(resp.alerts.is_empty());
    }

    #[tokio::test]
    async fn get_daily_alerts_propagates_error() {
        let service = build_service::<FailingDb>();
        let result = service.get_daily_alerts(10, 5).await;
        assert!(matches!(
            result,
            Err(GetDailyAlertsError::DatabaseError(msg)) if msg == "test error"
        ));
    }
}
