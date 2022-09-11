use super::command::{Command, Errors};
use crate::rust_analyzer::version::get;
use crate::services::downloader::Downloader;
use crate::services::versions::{Paging, Versions};
use futures::future::join_all;
use time::ext::NumericalDuration;
use time::format_description::FormatItem;
use time::parsing::Parsable;
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
}

fn compare_versions<T>(
    format: &T,
    current_version: &str,
    latest_version: &str,
) -> Result<bool, Errors>
where
    T: Parsable + ?Sized,
{
    let current_date = Date::parse(current_version, format)?;
    let current_date = current_date + 1.days();

    let latest_date = Date::parse(latest_version, format)?;

    if latest_date == current_date {
        Ok(false)
    } else {
        Ok(current_date < latest_date)
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

                let new_version_exists = compare_versions(
                    &self.date_format,
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

            let result = join_all(futures)
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

#[cfg(test)]
mod tests {
    use super::*;
    use time::format_description;

    #[test]
    fn test_compare_versions_equal_with_one_day_offset() {
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        let current_version = "2021-01-01";
        let latest_version = "2021-01-02";

        let result = compare_versions(&format, current_version, latest_version);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_compare_versions_return_true() {
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        let current_version = "2020-01-01";
        let latest_version = "2021-01-02";

        let result = compare_versions(&format, current_version, latest_version);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
