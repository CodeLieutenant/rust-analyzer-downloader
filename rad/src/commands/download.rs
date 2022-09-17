use tracing::debug;

use super::command::{Command, Errors};
use rust_analyzer_downloader::services::downloader::Downloader;
use std::fmt::Debug;

#[derive(Debug)]
pub(super) struct DownloadCommand {
    version: String,
    output: String,
    downloader: Downloader,
}

impl DownloadCommand {
    #[tracing::instrument]

    pub(super) fn new(version: String, output: String, downloader: Downloader) -> Self {
        Self {
            version,
            output,
            downloader,
        }
    }
}

#[async_trait::async_trait]
impl Command for DownloadCommand {
    #[tracing::instrument]
    async fn execute(self) -> Result<(), Errors> {
        debug!(
            version = &self.version,
            output = &self.output,
            "Downloading new version"
        );

        let result = self.downloader.download(&self.version, &self.output).await;

        debug!(
            version = &self.version,
            output = &self.output,
            "Version successfully downloaded from GitHub"
        );

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Errors::Download(err)),
        }
    }
}
