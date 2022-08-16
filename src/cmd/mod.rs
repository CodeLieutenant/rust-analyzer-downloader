use std::path::PathBuf;

use clap::{Parser, Subcommand};
use command::Command;
use directories::BaseDirs;

use self::{download::DownloadCommand, get_versions::GetVersionsCommand};

mod command;
mod download;
mod get_versions;

#[derive(Debug, Subcommand)]
enum Commands {
    Download {
        #[clap(required = false, value_parser, default_value_t=String::from("nightly"))]
        version: String,
        #[clap(required = false, value_parser, default_value_t=get_default_output_path())]
        output: String,
    },
    GetVersions {
        #[clap(required = false, value_parser, default_value_t = 3)]
        per_page: u32,
    },
}

#[derive(Debug, Parser)]
#[clap(name = "rust-analyzer-downloader",about = "Downloads and gets versions for Rust Analyzer", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

fn get_default_output_path() -> String {
    let base_dirs = BaseDirs::new().unwrap();
    let home_dir = base_dirs.home_dir();

    let mut buf = PathBuf::new();

    buf.push(home_dir);
    buf.push("bin");
    #[cfg(target_family = "windows")]
    buf.push("rust-analyzer.exe");
    #[cfg(target_family = "unix")]
    buf.push("rust-analyzer");

    buf.as_path().to_string_lossy().into()
}

pub async fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let future = match args.commands {
        Commands::Download { version, output } => {
            Box::pin(DownloadCommand::new(version, output).execute())
        }
        Commands::GetVersions { per_page } => Box::pin(GetVersionsCommand::new(per_page).execute()),
    };

    match future.await {
        Err(err) => Err(Box::new(err)),
        Ok(_) => Ok(())
    }
}
