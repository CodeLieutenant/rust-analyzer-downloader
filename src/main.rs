mod cmd;

use tracing::metadata::LevelFilter;
use tracing_appender::non_blocking;
use tracing_subscriber::{fmt::layer as fmt_layer, prelude::*, registry, filter::EnvFilter};

use cmd::execute;

fn main() {

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let (stdout_non_blocking, _stdout_guard) = non_blocking(std::io::stdout());
    let stdout_layer = fmt_layer()
        .with_ansi(true)
        .with_level(true)
        .with_thread_names(true)
        .with_target(true)
        .with_writer(stdout_non_blocking)
        .with_filter(LevelFilter::INFO);

    registry().with(env_filter).with(stdout_layer).init();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .thread_name("Executing Thread")
        .worker_threads(2)
        .build();

    let runtime = runtime.unwrap(); // TODO: Check error

    runtime.block_on(async {
        let result = execute().await;

        if let Err(e) = result {
            // TODO: Use logger
            eprintln!("Some error has occurred {e}");
        }
    });
}