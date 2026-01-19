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

    conf_service.validate_logging_config(logging_service.get_repo());
    logging_service.log_conf_validatation(conf_service.get_logging_config());

    logging_service.info(module_path!(), "EAS Weather Server ended");
}
