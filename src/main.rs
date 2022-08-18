
use rust_analyzer_downloader::cmd::execute;
use time::Instant;
use tokio::runtime::Builder;
use tracing::{error, info, metadata::LevelFilter};
use tracing_appender::non_blocking;
use tracing_subscriber::{filter::EnvFilter, fmt::layer as fmt_layer, prelude::*, registry};

fn main() {
    let start = Instant::now();

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

    let runtime = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .thread_name("Executing Thread")
        .worker_threads(2)
        .build();

    let runtime = match runtime {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            std::process::exit(1);
        }
    };

    runtime.block_on(async {
        let result = execute().await;

        if let Err(e) = result {
            error!("Some error has occurred {}", e);
        }
    });

    info!(
        "Command finished, exiting..., took {took}",
        took = start.elapsed()
    );
}
