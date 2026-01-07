use crate::core::domain::config::Config;

pub trait ConfigRepo {
    /// Generate and return the application configuration
    /// Should gather configuration based on the priority order of:
    /// CLI args > Environment File > Environment Variables > Configuration File > Default Configuration File > Default Values
    fn generate_config(&self) -> Config;

    /// Log any validation messages regarding the configuration
    /// This is needed to be triggered after the logging subsystem is initialized
    /// so that configutation log messages can be captured correctly.
    fn log_config_validation(&self, config: &Config);
}

pub trait ConfigCliRepo {
    /// Load configuration from CLI arguments
    fn load_config_from_cli(&self) -> Config;
}

pub trait ConfigEnvFileRepo {
    /// Set environment variables from file (e.g., .env file)
    /// This should called before loading configuration from environment variables
    /// as file will overwrite existing environment variables.
    fn set_env_from_file(&self);
}

pub trait ConfigEnvRepo {
    /// Load configuration from environment variables
    fn load_config_from_env(&self) -> Config;
}

pub trait ConfigFileRepo {
    /// Load configuration from a configuration file
    fn load_config_from_file(&self, file_path: &str) -> Config;
}
