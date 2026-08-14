use crate::domain::config::model::Config;
use crate::domain::config::port::ConfigPort;
use crate::domain::config::service::ConfigService;
use crate::domain::meta::port::MetaPort;
use crate::domain::meta::port::ValidatedConfig;

/// The MetaService provides access to application metadata, such as configuration data.
#[derive(Debug, Clone)]
pub struct MetaService<C>
where
    C: ConfigPort,
{
    conf_port: C,
}

/// The MetaService is responsible for providing access to application metadata, such as configuration data.
impl<C> MetaService<C>
where
    C: ConfigPort,
{
    pub fn new(conf_serv: ConfigService<C>) -> Self
    where
        C: ConfigPort,
    {
        let conf_port = conf_serv.get_port().clone();
        Self { conf_port }
    }

    /// Get the Config repository
    pub fn get_port(&self) -> &C {
        &self.conf_port
    }
}

/// Implement the MetaPort trait for MetaService, allowing it to provide access to configuration data.
impl<C> MetaPort for MetaService<C>
where
    C: ConfigPort,
{
    fn get_raw_config_data(&self) -> Config {
        self.conf_port.get_raw_config().clone()
    }

    // Return a validated configuration struct
    // which can be used by a handler
    fn get_conf(&self) -> ValidatedConfig {
        ValidatedConfig::new(
            self.conf_port.get_logging_config().clone(),
            self.conf_port.get_database_config().clone(),
            self.conf_port.get_webserver_config().clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::service::ConfigService;
    use crate::test_support::MockConfig;

    #[tokio::test]
    async fn get_conf_assembles_validated_config_from_port() {
        type MockConfigService = ConfigService<MockConfig>;
        let conf_serv: MockConfigService = ConfigService::new();
        let meta = MetaService::new(conf_serv);
        let conf = meta.get_conf();
        assert_eq!(
            serde_json::to_value(conf.get_logging_config()).unwrap(),
            serde_json::to_value(meta.get_port().get_logging_config()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(conf.get_database_config()).unwrap(),
            serde_json::to_value(meta.get_port().get_database_config()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(conf.get_webserver_config()).unwrap(),
            serde_json::to_value(meta.get_port().get_webserver_config()).unwrap()
        );
    }

    // `get_raw_config_data` requires MockConfig::get_raw_config, which is
    // intentionally unimplemented, so it is not unit-tested here.
}
