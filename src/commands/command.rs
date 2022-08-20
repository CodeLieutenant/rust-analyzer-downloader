use crate::services::downloader::Error as DownloaderError;
use crate::services::versions::Error as VersionsError;
use crate::rust_analyzer::version::Error as CurrentVersionError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub(super) enum Errors {
    #[error(transparent)]
    Download(#[from] DownloaderError),
    #[error(transparent)]
    GetVersions(#[from] VersionsError),
    #[error(transparent)]
    CurrentVersion(#[from] CurrentVersionError),
}

#[async_trait::async_trait]
pub(super) trait Command {
    async fn execute(self) -> Result<(), Errors>;
}
