use crate::domain::database::port::DatabasePort;
use crate::domain::meta::port::MetaPort;
use std::sync::Arc;

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
pub struct AppState<MR, DR>
where
    MR: MetaPort,
    DR: DatabasePort,
{
    meta_port: Arc<MR>,
    db_port: Arc<DR>,
}

impl<MR, DR> AppState<MR, DR>
where
    MR: MetaPort,
    DR: DatabasePort,
{
    /// Create a new AppState with the given services.
    pub fn new(meta_port: MR, db_port: DR) -> Self {
        Self {
            meta_port: Arc::new(meta_port),
            db_port: Arc::new(db_port),
        }
    }

    /// Get a reference to the MetaService.
    pub fn get_meta_port(&self) -> Arc<MR> {
        self.meta_port.clone()
    }

    /// Get a reference to the DatabaseService.
    pub fn get_db_port(&self) -> Arc<DR> {
        self.db_port.clone()
    }
}
