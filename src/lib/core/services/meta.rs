use crate::core::domain::meta::ports::Meta;
use crate::core::ports::inbound::config::ConfigRepo;
use crate::core::services::config::ConfigService;

#[derive(Debug, Clone)]
//pub struct MetaService<'a, C>
pub struct MetaService<C>
where
    C: ConfigRepo,
{
    //conf_repo: &'a C,
    conf_repo: C,
}

//impl<'a, C> MetaService<'a, C>
impl<C> MetaService<C>
where
    C: ConfigRepo,
{
    //pub fn new(conf_serv: &'a ConfigService<C>) -> Self
    pub fn new(conf_serv: ConfigService<C>) -> Self
    where
        C: ConfigRepo,
    {
        let conf_repo = conf_serv.get_repo().clone();
        Self { conf_repo }
    }

    /// Get the Config repository
    pub fn get_repo(&self) -> &C {
        &self.conf_repo
    }
}

impl<C> Meta for MetaService<C>
where
    C: ConfigRepo,
{
    async fn get_app_data(&self) -> String {
        // For demonstration purposes, we will just return a static string.
        // In a real application, this could be dynamic data such as version info, uptime, etc.
        "EAS Weather API - Version 1.0".to_string()
    }
}
