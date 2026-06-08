use crate::domain::alert::port::AlertPort;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::database::port::DatabasePort;

#[derive(Debug, Clone)]
pub struct DatabaseService<D>
where
    D: DatabasePort + AlertPort,
{
    db_port: D,
}

impl<D> DatabaseService<D>
where
    D: DatabasePort + AlertPort,
{
    /// Creates a new instance of DatabaseService.
    pub fn new<C>(conf_serv: &ConfigService<C>) -> Self
    where
        C: ConfigPort,
    {
        let conf_db = conf_serv.get_database_config();

        let db_port = D::new(conf_db);
        Self { db_port }
    }

    /// Get the Database port
    pub fn get_port(&self) -> &D {
        &self.db_port
    }

    /// Log configuration that's currently set
    pub fn log_adaptor_config<C>(&self, conf_serv: &ConfigService<C>)
    where
        C: ConfigPort,
    {
        let conf_db = conf_serv.get_database_config();

        self.db_port.log_adaptor_config(conf_db);
    }

    pub async fn create_pool<C>(&mut self, conf_serv: &ConfigService<C>)
    where
        C: ConfigPort,
    {
        let conf_db = conf_serv.get_database_config();
        // Implementation for creating database pool goes here

        self.db_port.create_pool(conf_db).await;
    }
}
