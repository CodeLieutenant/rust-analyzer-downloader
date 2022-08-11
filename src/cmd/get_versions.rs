#[derive(Debug, thiserror::Error)]
pub(super) enum Errors {}

#[derive(Debug)]
pub(super) struct GetVersionsCommand;

impl GetVersionsCommand {
    pub(super) fn new() -> Self {
        Self
    }

    pub(super) async fn execute(self) -> Result<(), Errors> {
        Ok(())
    }
}
