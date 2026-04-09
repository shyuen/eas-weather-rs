use crate::domain::database::port::DatabaseRepo;
use crate::domain::meta::port::MetaRepo;
use std::sync::Arc;

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
pub struct AppState<MR, DR>
where
    MR: MetaRepo,
    DR: DatabaseRepo,
{
    meta_repo: Arc<MR>,
    db_repo: Arc<DR>,
}

impl<MR, DR> AppState<MR, DR>
where
    MR: MetaRepo,
    DR: DatabaseRepo,
{
    /// Create a new AppState with the given services.
    pub fn new(meta_repo: MR, db_repo: DR) -> Self {
        Self {
            meta_repo: Arc::new(meta_repo),
            db_repo: Arc::new(db_repo),
        }
    }

    /// Get a reference to the MetaService.
    pub fn get_meta_repo(&self) -> Arc<MR> {
        self.meta_repo.clone()
    }
}
