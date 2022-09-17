use thiserror::Error as ThisError;

use rust_analyzer_downloader::rust_analyzer::version::Error as CurrentVersionError;
use rust_analyzer_downloader::services::downloader::Error as DownloaderError;
use rust_analyzer_downloader::services::versions::Error as VersionsError;

#[derive(Debug, ThisError)]
pub(super) enum Errors {
    #[error(transparent)]
    Download(#[from] DownloaderError),
    #[error(transparent)]
    GetVersions(#[from] VersionsError),
    #[error(transparent)]
    CurrentVersion(#[from] CurrentVersionError),

    #[error(transparent)]
    ParseDate(#[from] time::error::Parse),
}

#[async_trait::async_trait]
pub(super) trait Command {
    async fn execute(self) -> Result<(), Errors>;
}
