use tracing_appender::non_blocking;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger(path: &str, level: &str) -> non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(path, "xmu_assistant_bot");
    let (file_writer, guard) = non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .with_thread_ids(true)
        .with_target(true);

    let file_layer = fmt::layer().with_ansi(false).with_writer(file_writer);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    guard
}
