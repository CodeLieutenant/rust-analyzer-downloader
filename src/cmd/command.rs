use std::io::Error as IoError;

use reqwest::Error as ReqwestError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub(super) enum Errors {
    #[error(transparent)]
    Network(#[from] ReqwestError),

    #[error(transparent)]
    File(#[from] IoError),
}

#[async_trait::async_trait]
pub(super) trait Command {
    async fn execute(self) -> Result<(), Errors>;
}
