use crate::domain::config::model::Config;
use crate::domain::meta::service::ValidatedConfig;

//use std::future::Future;

pub trait MetaRepo: Clone + Send + Sync + 'static {
    /// Get application metadata, such as version info, uptime, etc.
    fn get_raw_config_data(&self) -> Config;

    /// Get the validated configuration struct which can be used by handlers
    fn get_conf(&self) -> ValidatedConfig;
}
