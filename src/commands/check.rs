use super::command::{Command, Errors};
use crate::rust_analyzer::version::get;
use crate::services::downloader::Downloader;
use crate::services::versions::{Paging, Versions};
use futures::future;
use tracing::{info, debug};

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
        // TODO: Parse versions as date and compare them
        current_version != latest_version
    }
}

#[async_trait::async_trait]
impl Command for CheckCommand {
    #[tracing::instrument]
    async fn execute(self) -> Result<(), Errors> {
        let current_version = get().await?;

        let version = self.versions.get(1, 2).await?;

        if let Paging::Next(_, data) = version {
            let futures = data.iter().map(|release| async {
                if !self.nightly && release.tag_name.as_str() == "nightly" {
                    debug!("nightly rust-analyzer is not enabled, skipping...");
                    return Ok(());
                }

                let same = self.compare_versions(
                    current_version.date_version.as_str(),
                    release.tag_name.as_str(),
                );

                if !same {
                    if self.should_download {
                        self.downloader
                            .download(release.tag_name.as_str(), self.output.as_str())
                            .await?;

                        info!(
                            "Downloaded version {} successfully downloaded",
                            &release.tag_name
                        );
                    } else {
                        info!("New version available: {}", release.tag_name);
                    }
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
