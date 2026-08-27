use std::process;

use sqlx::Connection;
use sqlx::mysql::MySqlConnection;

use eas_weather_rs::adaptors::clap::model::parse_cli;
use eas_weather_rs::adaptors::figment::model::ConfigFigment;
use eas_weather_rs::domain::config::port::ConfigPort;

/// Standalone migration runner for the eas-weather-rs database.
///
/// Uses the same config precedence as the main server:
///   CLI args → env vars → config file → defaults
///
/// Designed to run as a k8s init container before the main app starts.
#[tokio::main]
async fn main() {
    let cli = parse_cli();
    let config = ConfigFigment::with_cli(cli);
    let conn_url = config.get_database_config().conn_string.get().to_owned();

    println!("Running database migrations…");

    let mut conn = match MySqlConnection::connect(&conn_url).await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Failed to connect to database: {err}");
            process::exit(1);
        }
    };

    let migrations = sqlx::migrate!("./migrations");

    match migrations.run_direct(&mut conn).await {
        Ok(()) => {
            println!("Migrations complete");
        }
        Err(err) => {
            eprintln!("Migration failed: {err}");
            process::exit(1);
        }
    }
}
