use super::command::{Command, Errors};
use crate::rust_analyzer::version::get;
use crate::services::downloader::Downloader;
use crate::services::versions::{Paging, Versions};
use futures::future;
use time::ext::NumericalDuration;
use time::format_description::FormatItem;
use time::{format_description, Date};
use tracing::{debug, info, warn};

#[derive(Debug)]
pub(super) struct CheckCommand {
    output: String,
    should_download: bool,
    nightly: bool,
    downloader: Downloader,
    versions: Versions,
    date_format: Vec<FormatItem<'static>>,
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
            date_format: format_description::parse("[year]-[month]-[day]").unwrap(),
        }
    }

    fn compare_versions(
        &self,
        current_version: &str,
        latest_version: &str,
    ) -> Result<bool, Errors> {
        let current_date = Date::parse(current_version, &self.date_format)?;
        let current_date = current_date + 1.days();

        let latest_date = Date::parse(latest_version, &self.date_format)?;

        if latest_date == current_date {
            Ok(false)
        } else {
            Ok(current_date < latest_date)
        }
    }
}

#[async_trait::async_trait]
impl Command for CheckCommand {
    async fn execute(self) -> Result<(), Errors> {
        let current_version = get().await?;

        let version = self.versions.get(1, 2).await?;

        if let Paging::Next(_, data) = version {
            let futures = data.iter().map(|release| async {
                if !self.nightly && release.tag_name.as_str() == "nightly" {
                    debug!("nightly rust-analyzer is not enabled, skipping...");
                    return Ok(());
                }

                if self.nightly && release.tag_name.as_str() != "nightly" {
                    debug!(
                        "nightly rust-analyzer is enabled, skipping version {}...",
                        release.tag_name.as_str()
                    );
                    return Ok(());
                }

                let new_version_exists = self.compare_versions(
                    current_version.date_version.as_str(),
                    release.tag_name.as_str(),
                )?;

                if new_version_exists {
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

                info!("Current version is up to date");
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
