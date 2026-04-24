use clap::Parser;
use dotenv::dotenv;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use std::env;

use crate::adaptors::clap::model::Cli;
use crate::domain::config::model::Config;
use crate::domain::config::port::ConfigPort;
use crate::domain::database::model::Database;
use crate::domain::logging::model::Logging;
use crate::domain::logging::port::LoggingPort;
use crate::domain::webserver::model::Webserver;

#[derive(Debug, Clone)]
pub struct ConfigFigment {
    conf_raw: Config,
    conf_logging: Logging,
    conf_database: Database,
    conf_webserver: Webserver,
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

impl ConfigPort for ConfigFigment {
    fn new() -> Self {
        let conf = collect_raw_input();
        Self {
            conf_raw: conf.clone(),
            conf_logging: Logging::new(&conf.logging),
            conf_database: Database::new(&conf.database),
            conf_webserver: Webserver::new(&conf.webserver),
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

    /// Get the database configuration
    fn get_database_config(&self) -> &Database {
        &self.conf_database
    }

    /// Get the webserver configuration
    fn get_webserver_config(&self) -> &Webserver {
        &self.conf_webserver
    }

    // Log debug information regarding config inputs
    fn log_raw_config_input(&self, log_port: &impl LoggingPort) {
        // Log config from CLI
        log_port.debug(
            module_path!(),
            &format!("configuration from {:?}", Cli::parse()),
        );

        // Log config from ENV
        log_port.debug(
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
        log_port.debug(
            module_path!(),
            &format!("configuration from Files {:?}", conf_files),
        );

        // Log final raw config
        log_port.debug(
            module_path!(),
            &format!("final Raw Config {:?}", &self.conf_raw),
        );
    }

    /// Validate raw logging configuration
    fn log_raw_config_validation(&self, log_serv: &impl LoggingPort) {
        self.conf_logging
            .validate_raw_config(log_serv, &self.conf_raw.logging);
        self.conf_database
            .validate_raw_config(log_serv, &self.conf_raw.database);
        self.conf_webserver
            .validate_raw_config(log_serv, &self.conf_raw.webserver);
    }
}
