use std::{future::Future, io::Error as IoError};

use reqwest::Error as ReqwestError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub(super) enum Errors {
    #[error(transparent)]
    Network(#[from] ReqwestError),

    #[error(transparent)]
    File(#[from] IoError),
}

pub(super) trait Command {
    type Future: Future<Output = Result<(), Errors>>;

    fn execute(self) -> Self::Future;
}
