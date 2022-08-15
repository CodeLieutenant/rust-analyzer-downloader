#[derive(Debug, thiserror::Error)]
pub(super) enum Errors {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    File(#[from] std::io::Error),
}

pub(super) trait Command {
    type Future: std::future::Future<Output = Result<(), Errors>>;

    fn execute(self) -> Self::Future;
}
