use super::command::{Command, Errors};
use crate::services::downloader::Downloader;
use crate::services::versions::{Paging, Versions};
use futures::future;

#[derive(Debug)]
pub(super) struct CheckCommand {
    output: String,
    should_download: bool,
    nightly: bool,
    downloader: Downloader,
    versions: Versions,
}

impl CheckCommand {
    #[tracing::instrument]
    pub(super) fn new(
        output: String,
        downloader: Downloader,
        versions: Versions,
        should_download: bool,
        nightly: bool,
    ) -> Self {
        Self {
            output,
            downloader,
            versions,
            should_download,
            nightly,
        }
    }

    fn compare_versions(&self, current_version: &str, latest_version: &str) -> bool {
        // Parse versions as date and compare them
        current_version != latest_version
    }
}

#[async_trait::async_trait]
impl Command for CheckCommand {
    #[tracing::instrument]
    async fn execute(self) -> Result<(), Errors> {
        // TODO: Get Current Rust-Analyzer version
        let current_version = "2022-08-17";

        let version = self.versions.get(1, 2).await?;

        if let Paging::Next(_, data) = version {
            let latest_version = &data.get(0).unwrap().tag_name;

            let futures = data.iter().map(|release| async {
                if latest_version == "nightly" && self.nightly {
                    // TODO: Check the file which contains date when the last nightly version was downloaded
                    let should_download_based_on_last_download_date = false;

                    if self.should_download && should_download_based_on_last_download_date {
                        self.downloader.download(latest_version, &self.output).await?;
                        // TODO: Write the last check into file so that we know when to check again
                        // and when was the last check for nightly
                    }
                }

                let latest_version = &release.tag_name;

                if self.compare_versions(current_version, latest_version) && self.should_download {
                    self.downloader.download(latest_version, &self.output).await?;
                }

                Result::<(), Errors>::Ok(())
            });

            let result = future::join_all(futures)
                .await
                .drain(..)
                .find(|result| result.is_err());

            match result {
                Some(Err(err)) => Err(err),
                None | Some(Ok(_)) => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}
