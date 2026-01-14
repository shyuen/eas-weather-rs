use crate::core::ports::inbound::config::ConfigRepo;

#[derive(Debug, Clone)]
pub struct ConfigService<C>
where
    C: ConfigRepo,
{
    pub config: C,
}

impl<C> ConfigService<C>
where
    C: ConfigRepo,
{
    /// Creates a new instance of ConfigService.
    pub fn new() -> Self {
        let config = C::new();
        Self { config }
    }

    /// Logs debug information about the configuration inputs.
    pub fn log_debug_inputs(&self) {
        self.config.log_config_validation();
    }
}
