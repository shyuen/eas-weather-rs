use eas_weather_rs::adaptors::axum::model::WebserverAxum;
use eas_weather_rs::adaptors::figment::model::ConfigFigment;
use eas_weather_rs::adaptors::tracing::model::LoggingTracing;
use eas_weather_rs::adaptors::xsqlx::model::DatabaseMySql;
use eas_weather_rs::domain::config::service::ConfigService;
use eas_weather_rs::domain::database::service::DatabaseService;
use eas_weather_rs::domain::logging::service::LoggingService;
use eas_weather_rs::domain::meta::service::MetaService;
use eas_weather_rs::domain::webserver::service::WebserverService;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
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
    // let webserver_service: WebserverService<WebserverPoem> =
    //     WebserverService::new(&conf_service, &logging_service);
    let webserver_service: WebserverService<WebserverAxum> =
        WebserverService::new(&conf_service, &logging_service);

    // Output dataase adaptor configuration
    webserver_service.log_adaptor_config(&logging_service, &conf_service);

    let meta_service = MetaService::new(conf_service.clone());

    // Start Web Server
    webserver_service
        .start_server(
            &conf_service,
            &database_service,
            &logging_service,
            &meta_service,
        )
        .await
}
