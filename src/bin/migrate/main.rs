use std::collections::HashSet;
use std::process;

use sqlx::Connection;
use sqlx::Row;
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

    let mut migrations = sqlx::migrate!("./migrations");
    migrations.set_ignore_missing(true);

    // Collect applied migration versions from the database
    let applied: HashSet<i64> = sqlx::query("SELECT version FROM _sqlx_migrations")
        .fetch_all(&mut conn)
        .await
        .map(|rows| rows.iter().map(|r| r.get::<i64, _>("version")).collect())
        .unwrap_or_default();

    let total_up = migrations
        .iter()
        .filter(|m| m.migration_type.is_up_migration())
        .count();
    let pending_up: Vec<_> = migrations
        .iter()
        .filter(|m| m.migration_type.is_up_migration() && !applied.contains(&m.version))
        .collect();

    if pending_up.is_empty() {
        println!("All {} migration(s) already applied.", total_up);
    } else {
        println!("{} pending migration(s):", pending_up.len());
        for m in &pending_up {
            println!("  v{} — {}", m.version, m.description);
        }
    }

    match migrations.run(&mut conn).await {
        Ok(()) => {
            println!("Migrations complete");
        }
        Err(err) => {
            eprintln!("Migration failed: {err}");
            process::exit(1);
        }
    }
}
