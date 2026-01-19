use eas_weather_rs::adaptors::inbound::config::figment::FigmentConfig;
use eas_weather_rs::adaptors::outbound::logging::tracing::TracingLogging;
use eas_weather_rs::core::services::config::ConfigService;
use eas_weather_rs::core::services::logging::LoggingService;

fn main() {
    // Configuration
    let conf_service: ConfigService<FigmentConfig> = ConfigService::new();
    let logging_service: LoggingService<TracingLogging> =
        LoggingService::new(conf_service.get_logging_config());

    logging_service.info(module_path!(), "EAS Weather Server started");

    // Validate raw configurations
    conf_service.validate_raw_logging_config(logging_service.get_repo()); // Needs to be done after logger is initialized

    // Log the current configuration
    logging_service.log_set_config(conf_service.get_logging_config());

    logging_service.info(module_path!(), "EAS Weather Server ended");
}
