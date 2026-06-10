use std::sync::Once;

use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();

pub fn init_logging() {
    INIT.call_once(|| {
        let _ = fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive("flowlink=info".parse().unwrap()))
            .try_init();
    });
}

