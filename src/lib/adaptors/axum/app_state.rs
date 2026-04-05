use crate::domain::meta::port::MetaRepo;
use std::sync::Arc;

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
pub struct AppState<MR>
where
    MR: MetaRepo,
{
    meta_serv: Arc<MR>,
}

impl<MR> AppState<MR>
where
    MR: MetaRepo,
{
    /// Create a new AppState with the given services.
    pub fn new(meta_serv: MR) -> Self {
        Self {
            meta_serv: Arc::new(meta_serv),
        }
    }

    /// Get a reference to the MetaService.
    pub fn get_meta_serv(&self) -> Arc<MR> {
        self.meta_serv.clone()
    }
}
