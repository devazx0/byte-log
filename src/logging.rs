use crate::config::default_log_dir;
use crate::error::Result;
use std::fs;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging(verbose: bool, is_daemon: bool) -> Result<Option<WorkerGuard>> {
    let filter = if verbose {
        EnvFilter::new("bytelog=debug,info")
    } else {
        EnvFilter::new("bytelog=info")
    };

    if is_daemon {
        if let Ok(log_dir) = default_log_dir() {
            let _ = fs::create_dir_all(&log_dir);
            let file_appender = tracing_appender::rolling::daily(log_dir, "daemon.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
                .try_init()
                .ok();
            return Ok(Some(guard));
        }
    }

    if verbose {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_writer(std::io::stderr))
            .try_init()
            .ok();
    }

    Ok(None)
}
