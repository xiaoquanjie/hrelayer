use crate::configuration::{Configuration, server_name};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, fmt, registry};

/// 初始化日志
pub fn init(c: &Configuration, to_file: bool, to_console: bool) -> Option<WorkerGuard> {
    let (file, guard) = if to_file {
        let (b, g) = non_blocking(rolling::daily(&c.log.output, server_name()));
        (
            Some(
                fmt::layer()
                    .with_target(true)
                    .with_ansi(false)
                    .with_writer(b)
                    .with_filter(c.log.new_env_filter()),
            ),
            Some(g),
        )
    } else {
        (None, None)
    };

    match (file, to_console) {
        (Some(file), true) => registry()
            .with(file)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_filter(c.log.new_env_filter()),
            )
            .init(),
        (Some(file), false) => registry().with(file).init(),
        (None, true) => registry()
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_filter(c.log.new_env_filter()),
            )
            .init(),
        _ => {}
    }

    guard
}
