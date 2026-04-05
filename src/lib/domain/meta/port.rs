use crate::domain::config::model::Config;

//use std::future::Future;

pub trait MetaRepo: Clone + Send + Sync + 'static {
    /// Get application metadata, such as version info, uptime, etc.
    fn get_app_data(&self) -> Config;
}
