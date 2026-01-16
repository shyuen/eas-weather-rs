use crate::core::domain::config::Config;
use crate::core::ports::inbound::config::ConfigRepo;

#[derive(Debug, Clone)]
pub struct ConfigService<C>
where
    C: ConfigRepo,
{
    pub repo: C,
}

impl<C> ConfigService<C>
where
    C: ConfigRepo,
{
    /// Creates a new instance of ConfigService.
    pub fn new() -> Self {
        let repo = C::new();
        Self { repo }
    }

    /// Logs debug information about the configuration inputs.
    pub fn log_debug_inputs(&self) {
        self.repo.log_config_validation();
    }

    pub fn get_config(&self) -> &Config {
        self.repo.get_config()
    }
}
