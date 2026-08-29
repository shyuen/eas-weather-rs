use eas_weather_rs::adaptors::axum::model::WebserverAxum;
use eas_weather_rs::adaptors::clap::model::parse_cli_server;
use eas_weather_rs::adaptors::figment::model::ConfigFigment;
use eas_weather_rs::adaptors::tracing::model::LoggingTracing;
use eas_weather_rs::adaptors::xsqlx::model::DatabaseMySql;
use eas_weather_rs::domain::alert::service::AlertService;
use eas_weather_rs::domain::config::service::ConfigService;
use eas_weather_rs::domain::database::service::DatabaseService;
use eas_weather_rs::domain::logging::adaptor_logger::log_adaptor_config;
use eas_weather_rs::domain::logging::port::LoggingPort;
use eas_weather_rs::domain::webserver::port::WebserverStartError;
use eas_weather_rs::domain::webserver::service::WebserverService;

#[tokio::main]
async fn main() -> Result<(), WebserverStartError> {
    // Parse command-line arguments once and inject them into the config adaptor
    let conf_service: ConfigService<ConfigFigment> =
        // Create a new instance of ConfigService with the Figment adaptor
        ConfigService::from_config_port(ConfigFigment::with_cli(parse_cli_server())); // Inject parsed CLI arguments into the config adaptor

    // Initialize logging service with the loaded configuration so we can start logging messages
    let logging_port = LoggingTracing::new(conf_service.get_logging_config());

    // Output raw configuration information after logging service is initialized
    conf_service.log_raw_config_input();
    conf_service.log_raw_config_validation();

    // Output logging adaptor configuration
    log_adaptor_config(conf_service.get_logging_config(), &logging_port);

    // Initialize the database service
    let db_conf = conf_service.get_database_config().clone();
    let mut database_service: DatabaseService<DatabaseMySql> = DatabaseService::new(db_conf);

    // Output database adaptor configuration
    log_adaptor_config(
        conf_service.get_logging_config(),
        database_service.get_database_port(),
    );

    // Initialize database connection pool within the service
    if let Err(err) = database_service.create_pool().await {
        tracing::error!(error = %err, "create_pool: failed to establish database connection");
        std::process::exit(1);
    }

    // Start server to listen for incoming requests
    let webserver_service: WebserverService<WebserverAxum> =
        WebserverService::new(conf_service.get_webservicer_config());

    // Output webserver adaptor configuration
    log_adaptor_config(
        conf_service.get_logging_config(),
        webserver_service.get_webserver_port(),
    );

    // Initialize alert service
    let alert_service = AlertService::new(database_service.get_database_port());

    // Start Web Server
    // The webserver consumes the config service for the /conf endpoints
    webserver_service
        .start_server(
            &alert_service,
            &conf_service,
            database_service.get_database_port(),
        )
        .await
}
