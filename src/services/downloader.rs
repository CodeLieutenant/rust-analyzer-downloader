use async_compression::tokio::bufread::GzipDecoder;
use bytes::Bytes;
use directories::BaseDirs;
use futures_util::{Stream, StreamExt};
use reqwest::Error as ReqwestError;
use std::io::Error as IoError;
use std::{fmt::Debug, io::Cursor, path::PathBuf};
use thiserror::Error as ThisError;

#[cfg(target_family = "unix")]
use std::{fs::Permissions, os::unix::prelude::PermissionsExt};
use tokio::{
    fs::File,
    io::{AsyncWrite, BufReader},
};
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct Downloader {
    client: reqwest::Client,
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] ReqwestError),

    #[error(transparent)]
    File(#[from] IoError),
}

impl Downloader {
    #[tracing::instrument]
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    #[tracing::instrument]
    fn get_file_name(&self) -> &'static str {
        #[cfg(target_os = "windows")]
        #[cfg(target_arch = "x86_64")]
        return "rust-analyzer-x86_64-pc-windows-msvc.gz";

        #[cfg(target_os = "linux")]
        #[cfg(target_arch = "x86_64")]
        return "rust-analyzer-x86_64-unknown-linux-gnu.gz";

        #[cfg(target_os = "macos")]
        #[cfg(target_arch = "aarch64")]
        return "rust-analyzer-aarch64-apple-darwin.gz";

        #[cfg(target_os = "macos")]
        #[cfg(target_arch = "x86_64")]
        return "rust-analyzer-x86_64-apple-darwin.gz";
    }

    async fn decompress<S, O>(&self, stream: &mut S, output_file: &mut O) -> Result<(), Error>
    where
        S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
        O: AsyncWrite + Unpin,
    {
        let base_dirs = BaseDirs::new().unwrap();
        let mut temp_file_path = PathBuf::new();

        temp_file_path.push(base_dirs.cache_dir());
        temp_file_path.push("rust-analyzer.gz");
        debug!("Temp file path: {}", temp_file_path.display());

        let mut temp_file = File::create(&temp_file_path).await?;

        debug!("Copying Stream to Temp file");
        while let Some(chunk) = stream.next().await {
            let chunk_data: Bytes = chunk?; // TODO: Fix issue when this fails -> remove temporary file
            let mut cursor = Cursor::new(chunk_data);
            match tokio::io::copy(&mut cursor, &mut temp_file).await {
                Ok(_) => {
                    debug!(
                        "Copied chunk to temp file {temp_file}",
                        temp_file = temp_file_path.display()
                    );
                }
                Err(e) => {
                    error!("Some error has occurred while copying stream to temp file: {} TempFile {temp_file}", e, temp_file=temp_file_path.display());
                    tokio::fs::remove_file(&temp_file_path).await?;
                    return Err(Error::File(e));
                }
            }
        }
        debug!("Copying to TempFile finished");

        debug!("Starting decompression");
        let mut gzip_decoder = GzipDecoder::new(BufReader::new(File::open(&temp_file_path).await?));

        match tokio::io::copy(&mut gzip_decoder, output_file).await {
            Ok(_) => {
                debug!("Decompression finished, removing temp file");
                tokio::fs::remove_file(&temp_file_path).await?;
                Ok(())
            }
            Err(err) => {
                error!(
                    "Some error has occurred while decompressing: {} TempFile {temp_file}",
                    err,
                    temp_file = temp_file_path.display()
                );
                tokio::fs::remove_file(&temp_file_path).await?;
                Err(Error::File(err))
            }
        }
    }

    #[tracing::instrument]
    fn get_download_url(&self, version: &str) -> String {
        format!(
            "https://github.com/rust-lang/rust-analyzer/releases/download/{}/{}",
            version,
            self.get_file_name(),
        )
    }

    #[tracing::instrument]
    pub async fn download(&self, version: &str, output: &str) -> Result<(), Error> {
        let url = self.get_download_url(version);
        debug!("Downloading from: {url}", url = url);
        let res = self.client.get(url).send().await?;
        debug!("Response status: {status}", status = res.status());

        let mut stream = res.bytes_stream();
        let mut file = File::create(output).await?;

        #[cfg(target_family = "unix")]
        debug!("Setting permissions to file to 755 executable");
        #[cfg(target_family = "unix")]
        file.set_permissions(Permissions::from_mode(0o755)).await?;

        match self.decompress(&mut stream, &mut file).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tokio::fs::remove_file(output).await?;
                Err(e)
            }
        }
    }
}
