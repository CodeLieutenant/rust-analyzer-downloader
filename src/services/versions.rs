use reqwest::Error as ReqwestError;
use serde::Deserialize;
use std::io::Error as IoError;
use thiserror::Error as ThisError;
use tracing::{debug, trace};

const RELEASE_GITHUB_API_URL: &str =
    "https://api.github.com/repos/rust-lang/rust-analyzer/releases";
const PER_PAGE: &str = "per_page";

#[derive(Debug, Deserialize)]
pub struct ReleasesJsonResponse {
    pub name: String,
    pub tag_name: String,
    pub prerelease: bool,
}

#[derive(Debug)]
pub struct Versions {
    client: reqwest::Client,
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] ReqwestError),

    #[error(transparent)]
    File(#[from] IoError),
}

#[derive(Debug)]
pub enum Paging {
    Next(u32, Vec<ReleasesJsonResponse>),
    Done,
}

impl Versions {
    #[tracing::instrument]
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    #[tracing::instrument]
    pub async fn get(&self, page: u32, per_page: u32) -> Result<Paging, Error> {
        debug!("Sending request to {}", RELEASE_GITHUB_API_URL);
        let response = self
            .client
            .get(RELEASE_GITHUB_API_URL)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "rust-analyzer-downloader")
            .header("Accept-Encoding", "gzip")
            .header("Accept-Encoding", "deflate")
            .query(&[(PER_PAGE, per_page)])
            .send()
            .await?;

        trace!("Received response {:?}", response);
        let data = response.json::<Vec<ReleasesJsonResponse>>().await?;
        debug!("Versions: {:?}", data);

        if !data.is_empty() {
            Ok(Paging::Next(page + 1, data))
        } else {
            Ok(Paging::Done)
        }
    }
}
