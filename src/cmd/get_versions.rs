use serde::Deserialize;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub(super) enum Errors {
    #[error(transparent)]
    Network(#[from] reqwest::Error),
}

const RELEASE_GITHUB_API_URL: &str = "https://api.github.com/repos/rust-lang/rust-analyzer/releases";
const PER_PAGE: &str = "per_page";

#[derive(Debug, Deserialize)]
struct ReleasesJsonResponse {
    name: String,
    tag_name: String,
    prerelease: bool,
}

#[derive(Debug)]
pub(super) struct GetVersionsCommand {
    client: reqwest::Client,
    per_page: u32,
}

enum Paging {
    Next(u32),
    Done,
}

impl GetVersionsCommand {
    pub(super) fn new(per_page: u32) -> Self {
        Self {
            client: reqwest::Client::new(),
            per_page,
        }
    }

    async fn get_data<'a>(&self, url: &'a str, page: u32) -> Result<Paging, Errors> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "rust-analyzer-downloader")
            .header("Accept-Encoding", "gzip")
            .header("Accept-Encoding", "deflate")
            .query(&[(PER_PAGE, self.per_page)])
            .send()
            .await?;

        let data = response.json::<Vec<ReleasesJsonResponse>>().await?;

        if !data.is_empty() {
            data.iter().for_each(|release| {
                info!("Release {} <{}> Prerelease -> {}", release.name, release.tag_name, release.prerelease);
            });

            Ok(Paging::Next(page + 1))
        } else {
            Ok(Paging::Done)
        }
    }

    pub(super) async fn execute(self) -> Result<(), Errors> {
        match self.get_data(RELEASE_GITHUB_API_URL, 1).await {
            Ok(_next_page) => Ok(()),
            Err(err) => Err(err)
        }
    }
}
