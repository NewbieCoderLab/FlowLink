use std::{
    path::{Path, PathBuf},
    sync::Once,
};

use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();

pub fn init_logging(log_dir: Option<PathBuf>) {
    INIT.call_once(|| {
        let log_dir = log_dir.unwrap_or_else(|| PathBuf::from("logs"));
        init_logging_inner(&log_dir);
    });
}

fn init_logging_inner(log_dir: &Path) {
    let file_appender = tracing_appender::rolling::daily(log_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    Box::leak(Box::new(guard));

    let _ = fmt()
        .with_writer(non_blocking)
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("flowlink=info".parse().unwrap()),
        )
        .try_init();
}
