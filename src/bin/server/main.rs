use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use eas_weather_rs::adaptors::inbound::config::figment::FigmentConfig;
use eas_weather_rs::core::services::config::ConfigService;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .init();

    //let conf = FigmentConfig::new();
    //conf.log_config_validation();

    // We need to explicitly specify the type parameter here
    // because Rust cannot infer it automatically from a Service
    // with no parameters.
    let conf_service: ConfigService<FigmentConfig> = ConfigService::new();
    conf_service.log_debug_inputs();

    println!("Hello, world!");
}
