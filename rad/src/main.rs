use time::Instant;
use tokio::runtime::Builder;
use tracing::{debug, error, metadata::LevelFilter};
use tracing_subscriber::{filter::EnvFilter, fmt::layer as fmt_layer, prelude::*, registry};

mod commands;

use crate::commands::execute;

fn main() {
    let start = Instant::now();

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let stdout_layer = fmt_layer()
        .with_ansi(true)
        .with_level(true)
        .with_thread_names(false)
        .with_target(false)
        .with_writer(std::io::stdout);

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
            error!("Some error has occurred: {}", e);
        }
    });

    debug!(
        "Command finished, exiting..., took {took}",
        took = start.elapsed()
    );
}
