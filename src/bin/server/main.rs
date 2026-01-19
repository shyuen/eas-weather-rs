use eas_weather_rs::adaptors::inbound::config::figment::FigmentConfig;
use eas_weather_rs::adaptors::outbound::logging::tracing::TracingLogging;
use eas_weather_rs::core::services::config::ConfigService;
use eas_weather_rs::core::services::logging::LoggingService;

fn main() {
    // Initialize configuration service to load configuration input
    let conf_service: ConfigService<FigmentConfig> = ConfigService::new();

    // Initialize logging service with the loaded configuration so we can start logging messages
    let logging_service: LoggingService<TracingLogging> =
        LoggingService::new(conf_service.get_logging_config());

    // TODO: Implement cool ASCII art banner
    //logging_service.info(module_path!(), "EAS Weather Server started");

    // Output configuration information after logger is initialized
    conf_service.log_raw_config_input(logging_service.get_repo()); // Output raw config inputs with debug level set, needs to be done after logger is initialized
    conf_service.validate_raw_logging_config(logging_service.get_repo()); // Validate raw configurations, needs to be done after logger is initialized

    // Output configuration information
    logging_service.log_set_config(conf_service.get_logging_config());

    //logging_service.info(module_path!(), "EAS Weather Server ended");
}
