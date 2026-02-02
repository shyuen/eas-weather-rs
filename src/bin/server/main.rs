use eas_weather_rs::adaptors::inbound::config::figment::ConfigFigment;
use eas_weather_rs::adaptors::outbound::logging::tracing::LoggingTracing;
use eas_weather_rs::adaptors::outbound::mysql::xsqlx::DatabaseMySql;
use eas_weather_rs::core::services::config::ConfigService;
use eas_weather_rs::core::services::database::DatabaseService;
use eas_weather_rs::core::services::logging::LoggingService;

#[tokio::main]
async fn main() {
    // Initialize configuration service to load configuration input
    let conf_service: ConfigService<ConfigFigment> = ConfigService::new();

    // Initialize logging service with the loaded configuration so we can start logging messages
    let logging_service: LoggingService<LoggingTracing> = LoggingService::new(&conf_service);

    // Output raw configuration information after logging service is initialized
    conf_service.log_raw_config_input(&logging_service); // Output raw config inputs with debug level set
    conf_service.log_raw_config_validation(&logging_service); // Validate raw configurations

    // Output logging adaptor configuration
    logging_service.log_adaptor_config(&conf_service);

    // Initialize the database service
    let mut database_service: DatabaseService<DatabaseMySql> =
        DatabaseService::new(&conf_service, &logging_service);

    // Output dataase adaptor configuration
    database_service.log_adaptor_config(&logging_service, &conf_service);

    // Initialize database connection pool within the service
    database_service
        .create_pool(&conf_service, &logging_service)
        .await;

    // TODO: Initialize other services (e.g., weather data service, API service, etc.)

    // TODO: Start server to listen for incoming requests
}
