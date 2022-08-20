use super::{command::Errors, Command};
use crate::services::versions::{Paging, Versions};
use tracing::info;

#[derive(Debug)]
pub(super) struct GetVersionsCommand {
    versions: Versions,
    per_page: u32,
}

impl GetVersionsCommand {
    #[tracing::instrument]
    pub(super) fn new(versions: Versions, per_page: u32) -> Self {
        Self { versions, per_page }
    }
}

#[async_trait::async_trait]
impl Command for GetVersionsCommand {
    async fn execute(self) -> Result<(), Errors> {
        let result = self.versions.get(1, self.per_page).await;

        match result {
            Ok(Paging::Next(_next_page, data)) => {
                data.iter().for_each(|release| {
                    info!("Version: {} Is Prerelease: {}", &release.tag_name, release.prerelease);
                });

                Ok(())
            }
            Err(err) => Err(Errors::GetVersions(err)),
            _ => Ok(()),
        }
    }
}
