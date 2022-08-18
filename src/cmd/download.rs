use super::command::{Command, Errors};
use crate::services::downloader::Downloader;
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
        let result = self.downloader.download(&self.version, &self.output).await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Errors::Download(err)),
        }
    }
}
