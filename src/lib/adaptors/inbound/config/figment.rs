use clap::Parser;
use dotenv::dotenv;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use std::env;

use crate::adaptors::inbound::config::clap::Cli;
use crate::core::domain::config::logging::Logging;
use crate::core::domain::config::raw::Config;
use crate::core::ports::inbound::config::ConfigRepo;
use crate::core::ports::outbound::logging::LoggingRepo;

#[derive(Debug)]
pub struct FigmentConfig {
    conf_raw: Config,
    conf_logging: Logging,
}

fn collect_raw_input() -> Config {
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
            std::process::exit(1); // We use exit for planned exits instead of panics
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
        .join(Env::prefixed("LOGGING__").split("__"))
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

    conf
}

impl ConfigRepo for FigmentConfig {
    fn new() -> Self {
        let conf = collect_raw_input();
        Self {
            conf_raw: conf.clone(),
            conf_logging: Logging::new(&conf.logging),
        }
    }

    /// Get the raw configuration
    fn get_raw_config(&self) -> &Config {
        &self.conf_raw
    }

    /// Get the logging configuration
    fn get_logging_config(&self) -> &Logging {
        &self.conf_logging
    }

    // Log debug information regarding config inputs
    fn log_raw_config_input(&self, log_repo: &impl LoggingRepo) {
        // Log config from CLI
        log_repo.debug(
            module_path!(),
            &format!("configuration from {:?}", Cli::parse()),
        );

        // Log config from ENV
        log_repo.debug(
            module_path!(),
            &format!(
                "configuration from Env {:?}",
                env::vars()
                    .filter(|(k, _)| k.starts_with("LOGGING__")
                        || k.starts_with("SERVER__")
                        || k.starts_with("DATABASE__"))
                    .map(|(k, v)| (k.replace("__", "."), v))
                    .collect::<Vec<_>>()
            ),
        );

        let conf_files: Config = match Figment::new()
            .join(Toml::file(
                &self
                    .conf_raw
                    .config_file
                    .clone()
                    .unwrap_or("config/config.toml".to_string()),
            ))
            .join(Toml::file("./config/default.toml"))
            .extract()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error extracting configuration: {}", e);
                std::process::exit(1);
            }
        };
        log_repo.debug(
            module_path!(),
            &format!("configuration from Files {:?}", conf_files),
        );

        // Log final raw config
        log_repo.debug(
            module_path!(),
            &format!("final Raw Config {:?}", &self.conf_raw),
        );
    }

    /// Validate raw logging configuration
    fn log_raw_config_validation(&self, log_serv: &impl LoggingRepo) {
        self.conf_logging
            .validate_raw_logging_config(log_serv, &self.conf_raw.logging);
    }
}
