use std::collections::HashSet;
use std::process;

use sqlx::Connection;
use sqlx::Row;
use sqlx::mysql::MySqlConnection;

use eas_weather_rs::adaptors::clap::model::parse_cli;
use eas_weather_rs::adaptors::figment::model::ConfigFigment;
use eas_weather_rs::adaptors::tracing::model::LoggingTracing;
use eas_weather_rs::domain::config::port::ConfigPort;
use eas_weather_rs::domain::logging::port::LoggingPort;

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

    // Initialize logging from config so container logs honour the format
    // (text/json) and trace_level used by the rest of the app.
    let _logging_port = LoggingTracing::new(config.get_logging_config());

    let conn_url = config.get_database_config().conn_string.get().to_owned();

    tracing::info!("Running database migrations…");

    let mut conn = match MySqlConnection::connect(&conn_url).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(
                error_code = "database_connection_error",
                message = %err,
                "Failed to connect to database"
            );
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
        tracing::info!("All {} migration(s) already applied.", total_up);
    } else {
        tracing::info!("{} pending migration(s):", pending_up.len());
        for m in &pending_up {
            tracing::info!("  v{} — {}", m.version, m.description);
        }
    }

    match migrations.run(&mut conn).await {
        Ok(()) => {
            tracing::info!("Migrations complete");
        }
        Err(err) => {
            tracing::error!(
                error_code = "migration_failed",
                message = %err,
                "Migration failed"
            );
            process::exit(1);
        }
    }
}
