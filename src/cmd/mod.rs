use std::path::PathBuf;

use clap::{Parser, Subcommand};
use directories::BaseDirs;

mod download;
mod get_versions;

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(arg_required_else_help = true)]
    Download {
        #[clap(required = true, value_parser)]
        version: String,
        #[clap(required = false, value_parser)]
        output: Option<String>,
    },
    GetVersions,
}

#[derive(Debug, Parser)]
#[clap(name = "rust-analyzer-downloader")]
#[clap(about = "Downloads and gets versions for Rust Analyzer", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

pub async fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let base_dirs = BaseDirs::new().unwrap();
    let home_dir = base_dirs.home_dir();

    match args.commands {
        Commands::Download { version, output } => {
            let output_path = match output {
                Some(value) => value,
                None => {
                    let mut buf = PathBuf::new();
                    buf.push(home_dir);
                        buf.push("bin");
                        buf.push("rust-analyzer");

                    buf.as_path().to_string_lossy().into()
                },
            };

            let command = download::DownloadCommand::new(version, output_path);
            if let Err(e) = command.execute().await {
                Err(Box::new(e))
            } else {
                Ok(())
            }
        }
        Commands::GetVersions => {
            let command = get_versions::GetVersionsCommand::new();

            if let Err(e) = command.execute().await {
                Err(Box::new(e))
            } else {
                Ok(())
            }
        }
    }
}
