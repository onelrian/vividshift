use anyhow::Result;
use std::str::FromStr;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

use crate::config::AppConfig;

pub fn init_logging(config: &AppConfig) -> Result<Option<WorkerGuard>> {
    let log_level = config.logging.level.as_str();
    
    // Create base filter
    let filter = EnvFilter::from_str(log_level)
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let mut guard = None;

    if config.logging.file_enabled {
        // Ensure log directory exists
        if let Some(parent) = std::path::Path::new(&config.logging.file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file_appender = tracing_appender::rolling::daily(
            std::path::Path::new(&config.logging.file_path)
                .parent()
                .unwrap_or_else(|| std::path::Path::new(".")),
            std::path::Path::new(&config.logging.file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("app.log"),
        );

        let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);
        guard = Some(worker_guard);

        if config.logging.json_format {
            // JSON format for production
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_writer(std::io::stdout)
                        .with_span_events(FmtSpan::CLOSE),
                )
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_writer(non_blocking)
                        .with_span_events(FmtSpan::CLOSE),
                )
                .init();
        } else {
            // Pretty format for development
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_writer(std::io::stdout)
                        .with_span_events(FmtSpan::CLOSE),
                )
                .with(
                    fmt::layer()
                        .with_writer(non_blocking)
                        .with_span_events(FmtSpan::CLOSE),
                )
                .init();
        }
    } else {
        // Console only
        if config.logging.json_format {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_writer(std::io::stdout)
                        .with_span_events(FmtSpan::CLOSE),
                )
                .init();
        } else {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_writer(std::io::stdout)
                        .with_span_events(FmtSpan::CLOSE),
                )
                .init();
        }
    }

    Ok(guard)
}
