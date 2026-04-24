use crate::domain::alert::port::DatabasePortAlert;
use crate::domain::database::port::DatabasePort;
use crate::domain::database::service::DatabaseService;
use crate::domain::logging::port::LoggingPort;
use crate::domain::logging::service::LoggingService;

/// The AlertService struct provides methods for managing alerts.
/// Contains ports to interact with other services
#[derive(Debug, Clone)]
pub struct AlertService<L, D>
where
    L: LoggingPort,
    D: DatabasePortAlert + DatabasePort,
{
    log_port: L,
    db_port: D,
}

impl<L, D> AlertService<L, D>
where
    L: LoggingPort,
    D: DatabasePortAlert + DatabasePort,
{
    /// Creates a new instance of AlertService.
    pub fn new(log_serv: &LoggingService<L>, db_serv: DatabaseService<D>) -> Self
    where
        L: LoggingPort,
        D: DatabasePortAlert + DatabasePort,
    {
        let log_port = log_serv.get_port();
        let db_port = db_serv.get_port();

        // Log the initialization of the AlertService
        log_port.info(module_path!(), "Initializing AlertService");

        Self {
            log_port: log_port.clone(),
            db_port: db_port.clone(),
        }
    }

    /// Get the Database repository
    pub fn get_db_port(&self) -> &D {
        &self.db_port
    }
}
