use eas_weather_rs::adaptors::axum::model::WebserverAxum;
use eas_weather_rs::adaptors::figment::model::ConfigFigment;
use eas_weather_rs::adaptors::tracing::model::LoggingTracing;
use eas_weather_rs::adaptors::xsqlx::model::DatabaseMySql;
use eas_weather_rs::domain::alert::service::AlertService;
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
    conf_service.log_raw_config_input(); // Output raw config inputs with debug level set
    conf_service.log_raw_config_validation(); // Validate raw configurations

    // Output logging adaptor configuration
    conf_service.log_adaptor_config(logging_service.get_port());

    // Initialize the database service
    let mut database_service: DatabaseService<DatabaseMySql> = DatabaseService::new(&conf_service);

    // Output database adaptor configuration
    conf_service.log_adaptor_config(database_service.get_port());

    // Show the raw configuration in the logs after logging service is initialized
    //conf_service.emit_raw_config(&logging_service);

    // Initialize database connection pool within the service
    if let Err(err) = database_service.create_pool().await {
        tracing::error!(error = %err, "create_pool: failed to establish database connection");
        std::process::exit(1);
    }

    // Start server to listen for incoming requests
    let webserver_service: WebserverService<WebserverAxum> = WebserverService::new(&conf_service);

    // Output webserver adaptor configuration
    conf_service.log_adaptor_config(webserver_service.get_port());

    // Initialize meta service
    let meta_service = MetaService::new(conf_service.clone());
    //let meta_service = MetaService::new(&conf_service);

    // Initalize alert service
    let alert_service = AlertService::new(database_service.clone());

    // Start Web Server
    webserver_service
        .start_server(&conf_service, &alert_service, &meta_service)
        .await
}
