use std::{
    path::{Path, PathBuf},
    sync::Once,
};

use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();
const LOG_FILE_NAME: &str = "app.log";

pub fn init_logging(log_dir: Option<PathBuf>) {
    INIT.call_once(|| {
        let log_dir = log_dir.unwrap_or_else(|| PathBuf::from("logs"));
        init_logging_inner(&log_dir);
    });
}

fn init_logging_inner(log_dir: &Path) {
    let file_appender = daily_file_appender(log_dir);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    Box::leak(Box::new(guard));

    let _ = fmt()
        .with_writer(non_blocking)
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("flowlink=info".parse().unwrap()),
        )
        .try_init();
}

pub(crate) fn daily_file_appender(
    log_dir: &Path,
) -> tracing_appender::rolling::RollingFileAppender {
    tracing_appender::rolling::daily(log_dir, LOG_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn daily_file_appender_writes_app_log_in_target_directory() {
        let dir = tempdir().expect("tempdir");
        let mut appender = daily_file_appender(dir.path());

        writeln!(appender, "flowlink test log").expect("write log");
        appender.flush().expect("flush log");

        let log_files = fs::read_dir(dir.path())
            .expect("read log dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("app.log"))
            .collect::<Vec<_>>();

        assert_eq!(log_files.len(), 1);
        let content = fs::read_to_string(log_files[0].path()).expect("read log file");
        assert!(content.contains("flowlink test log"));
    }

    #[test]
    fn init_logging_can_be_called_more_than_once() {
        let first = tempdir().expect("first tempdir");
        let second = tempdir().expect("second tempdir");

        init_logging(Some(first.path().to_path_buf()));
        init_logging(Some(second.path().to_path_buf()));
    }
}
