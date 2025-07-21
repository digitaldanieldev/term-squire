use anyhow::{Error, Result};
use color_eyre::Report;
use tracing::{debug, error, info, trace, warn, Level};
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
};

pub fn load_logging_config(log_level: Level) -> Result<(), Report> {
    trace!("Entering load_logging_config function");
    color_eyre::install()?;
    let filter = EnvFilter::new(format!("my_crate={}", log_level.as_str()));
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_max_level(log_level)
        .with_span_events(FmtSpan::ACTIVE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    info!(
        "Logging configuration successfully applied at level: {:?}",
        log_level
    );
    trace!("Exiting load_logging_config function");
    Ok(())
}

pub fn parse_log_level(log_level: &str) -> Result<Level, Error> {
    trace!("Parsing log level: {}", log_level);
    match log_level.to_lowercase().as_str() {
        "error" => Ok(Level::ERROR),
        "warn" => Ok(Level::WARN),
        "info" => Ok(Level::INFO),
        "debug" => Ok(Level::DEBUG),
        "trace" => Ok(Level::TRACE),
        _ => {
            warn!("Unrecognized log level, defaulting to INFO.");
            Ok(Level::INFO)
        }
    }
}
