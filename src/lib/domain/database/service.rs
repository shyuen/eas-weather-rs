use crate::domain::alert::port::DatabasePortAlert;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;
use crate::domain::logging::port::LoggingPort;
use crate::domain::logging::service::LoggingService;

#[derive(Debug, Clone)]
pub struct DatabaseService<D>
where
    D: DatabasePort + DatabasePortAlert,
{
    repo: D,
}

impl<D> DatabaseService<D>
where
    D: DatabasePort + DatabasePortAlert,
{
    /// Creates a new instance of DatabaseService.
    pub fn new<C, L>(conf_serv: &ConfigService<C>, log_serv: &LoggingService<L>) -> Self
    where
        C: ConfigPort,
        L: LoggingPort,
    {
        let log_port = log_serv.get_port();
        let conf_db = conf_serv.get_database_config();

        let repo = D::new(log_port, conf_db);
        Self { repo }
    }

    /// Get the Database repository
    pub fn get_port(&self) -> &D {
        &self.repo
    }

    /// Log configuration that's currently set
    pub fn log_adaptor_config<L, C>(
        &self,
        log_serv: &LoggingService<L>,
        conf_serv: &ConfigService<C>,
    ) where
        C: ConfigPort,
        L: LoggingPort,
    {
        let log_port = log_serv.get_port();
        let conf_db = conf_serv.get_database_config();

        self.repo.log_adaptor_config(log_port, conf_db);
    }

    pub async fn create_pool<C, L>(
        &mut self,
        conf_serv: &ConfigService<C>,
        log_serv: &LoggingService<L>,
    ) where
        C: ConfigPort,
        L: LoggingPort,
    {
        let log_port = log_serv.get_port();
        let conf_db = conf_serv.get_database_config();
        // Implementation for creating database pool goes here

        self.repo.create_pool(log_port, conf_db).await;
    }
}
