use clap::Parser;
use dotenv::dotenv;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use std::env;
use tracing::debug;
use tracing::info;

use crate::core::domain::config::Config;

use crate::core::ports::inbound::config::ConfigRepo;

use crate::adaptors::inbound::config::clap::Cli;

#[derive(Debug)]
pub struct FigmentConfig {
    config: Config,
}

impl FigmentConfig {
    pub fn new() -> Self {
        // This line loads the environment variables from the ".env" file.
        dotenv().ok();

        // Use Figment to load preliminary configuration
        // We need this to handle the certain configuration options such as using a
        // different configuration file to load when dictated by the CLI or ENV.
        let conf: Config = match Figment::new()
            .join(Serialized::defaults(Cli::parse()))
            .join(Env::prefixed("APP__").split("__"))
            .extract()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error extracting configuration: {}", e);
                std::process::exit(1);
            }
        };

        // Check which selected config file to load from CLI or ENV
        let config_file = match conf.config_file {
            Some(x) => x.to_string(),
            None => "config/config.toml".to_string(),
        };

        // Use Figment to load configuration from multiple sources the following priority
        // CLI > ENV > FILE > DEFAULT FILE > CODE
        let conf: Config = match Figment::new()
            .join(Serialized::defaults(Cli::parse()))
            .join(Env::prefixed("APP__").split("__"))
            .join(Env::prefixed("SERVER__").split("__"))
            .join(Env::prefixed("DATABASE__").split("__"))
            .join(Toml::file(config_file))
            .join(Toml::file("./config/default.toml"))
            .extract()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error extracting configuration: {}", e);
                std::process::exit(1);
            }
        };

        Self { config: conf }
    }

    pub fn log_config_validation(&self) {
        // Log debug information regarding config inputs
        debug!("configuration from {:?}", Cli::parse());
        debug!(
            "configuration from Env {:?}",
            env::vars()
                .filter(|(k, _)| k.starts_with("APP__") || k.starts_with("SERVER__"))
                .map(|(k, v)| (k.replace("__", "."), v))
                .collect::<Vec<_>>()
        );
        info!("final configuration output {:?}", &self.config);
    }
}

impl ConfigRepo for FigmentConfig {
    fn new() -> Self {
        // This line loads the environment variables from the ".env" file.
        dotenv().ok();

        // Use Figment to load preliminary configuration
        // We need this to handle the certain configuration options such as using a
        // different configuration file to load when dictated by the CLI or ENV.
        let conf: Config = match Figment::new()
            .join(Serialized::defaults(Cli::parse()))
            .join(Env::prefixed("APP__").split("__"))
            .extract()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error extracting configuration: {}", e);
                std::process::exit(1);
            }
        };

        // Check which selected config file to load from CLI or ENV
        let config_file = match conf.config_file {
            Some(x) => x.to_string(),
            None => "config/config.toml".to_string(),
        };

        // Use Figment to load configuration from multiple sources the following priority
        // CLI > ENV > FILE > DEFAULT FILE > CODE
        let conf: Config = match Figment::new()
            .join(Serialized::defaults(Cli::parse()))
            .join(Env::prefixed("APP__").split("__"))
            .join(Env::prefixed("SERVER__").split("__"))
            .join(Env::prefixed("DATABASE__").split("__"))
            .join(Toml::file(config_file))
            .join(Toml::file("./config/default.toml"))
            .extract()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error extracting configuration: {}", e);
                std::process::exit(1);
            }
        };

        Self { config: conf }
    }

    fn log_config_validation(&self) {
        // Log debug information regarding config inputs
        debug!("configuration from {:?}", Cli::parse());
        debug!(
            "configuration from Env {:?}",
            env::vars()
                .filter(|(k, _)| k.starts_with("APP__") || k.starts_with("SERVER__"))
                .map(|(k, v)| (k.replace("__", "."), v))
                .collect::<Vec<_>>()
        );
        info!("final configuration output {:?}", &self.config);
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}
