use crate::core::ports::inbound::config::ConfigRepo;
use crate::core::ports::outbound::database::DatabaseRepo;
use crate::core::ports::outbound::logging::LoggingRepo;
use crate::core::services::config::ConfigService;
use crate::core::services::logging::LoggingService;

#[derive(Debug, Clone)]
pub struct DatabaseService<D>
where
    D: DatabaseRepo,
{
    pub repo: D,
}

impl<D> DatabaseService<D>
where
    D: DatabaseRepo,
{
    /// Creates a new instance of DatabaseService.
    pub fn new<C, L>(conf_serv: &ConfigService<C>, log_serv: &LoggingService<L>) -> Self
    where
        C: ConfigRepo,
        L: LoggingRepo,
    {
        let log_repo = &log_serv.repo;
        let conf_db = conf_serv.get_database_config();

        let repo = D::new(log_repo, conf_db);
        Self { repo }
    }

    /// Get the Database repository
    pub fn get_repo(&self) -> &D {
        &self.repo
    }

    /// Log configuration that's currently set
    pub fn log_adaptor_config<L, C>(
        &self,
        log_serv: &LoggingService<L>,
        conf_serv: &ConfigService<C>,
    ) where
        C: ConfigRepo,
        L: LoggingRepo,
    {
        let log_repo = log_serv.get_repo();
        let conf_db = conf_serv.get_database_config();

        self.repo.log_adaptor_config(log_repo, conf_db);
    }

    pub async fn create_pool<C, L>(
        &mut self,
        conf_serv: &ConfigService<C>,
        log_serv: &LoggingService<L>,
    ) where
        C: ConfigRepo,
        L: LoggingRepo + Sync,
    {
        let log_repo = log_serv.get_repo();
        let conf_db = conf_serv.get_database_config();
        // Implementation for creating database pool goes here

        self.repo.create_pool(log_repo, conf_db).await;
    }
}
