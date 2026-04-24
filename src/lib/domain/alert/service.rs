use crate::domain::alert::port::DatabasePortAlert;
use crate::domain::database::port::DatabasePort;
use crate::domain::database::service::DatabaseService;
use crate::domain::logging::port::LoggingRepo;
use crate::domain::logging::service::LoggingService;

/// The AlertService struct provides methods for managing alerts.
/// Contains ports to interact with other services
#[derive(Debug, Clone)]
pub struct AlertService<D>
where
    D: DatabasePortAlert + DatabasePort,
{
    db_port: D,
}

impl<D> AlertService<D>
where
    D: DatabasePortAlert + DatabasePort,
{
    /// Creates a new instance of AlertService.
    pub fn new<L>(log_serv: &LoggingService<L>, db_serv: DatabaseService<D>) -> Self
    where
        L: LoggingRepo,
    {
        let log_repo = &log_serv.repo;
        let db_port = db_serv.get_repo();

        // Log the initialization of the AlertService
        log_repo.info(module_path!(), "Initializing AlertService");

        Self {
            db_port: db_port.clone(),
        }
    }

    /// Get the Database repository
    pub fn get_db_port(&self) -> &D {
        &self.db_port
    }
}
