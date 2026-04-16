use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const APP_NAME: &str = "Murmur";

pub fn init_logging(
    log_dir: &PathBuf,
    foreground: bool,
) -> tracing_appender::non_blocking::WorkerGuard {
    std::fs::create_dir_all(log_dir).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "murmur.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = fmt::layer().with_writer(non_blocking).with_ansi(false);

    if foreground {
        let stderr_layer = fmt::layer().with_writer(std::io::stderr);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stderr_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    guard
}

fn send_notification(subtitle: &str, message: &str) {
    if let Err(e) = mac_notification_sys::send_notification(APP_NAME, Some(subtitle), message, None)
    {
        tracing::warn!("Failed to send notification: {}", e);
    }
}

pub fn notify_info(message: &str) {
    tracing::info!("notify_info: {}", message);
    send_notification("", message);
}

pub fn notify_error(message: &str) {
    tracing::error!("notify_error: {}", message);
    send_notification("Error", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_creates_log_dir() {
        let tmp = std::env::temp_dir().join(format!("murmur_test_log_{}", std::process::id()));
        let _guard = init_logging(&tmp, false);
        assert!(tmp.exists());
        std::fs::remove_dir_all(&tmp).ok();
    }
}
