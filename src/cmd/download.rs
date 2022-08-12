use async_compression::tokio::bufread::GzipDecoder;
use bytes::Bytes;
use directories::BaseDirs;
use futures_util::{Stream, StreamExt};
#[cfg(target_family="unix")]
use std::{os::unix::prelude::PermissionsExt, fs::Permissions};
use std::{
    fmt::Debug,  io::Cursor, path::PathBuf,
};
use tokio::{
    fs::File,
    io::{self, AsyncWrite, BufReader},
};
use tracing::{debug, error, info, warn, span, Level};

#[derive(Debug, thiserror::Error)]
pub(super) enum Errors {
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    #[error(transparent)]
    File(#[from] io::Error),
}

#[derive(Debug)]
pub(super) struct DownloadCommand {
    version: String,
    output: String,
    client: reqwest::Client,
}

impl DownloadCommand {
    #[tracing::instrument]
    pub(super) fn new(version: String, output: String) -> Self {
        Self {
            version,
            output,
            client: reqwest::Client::new(),
        }
    }

    async fn decompress<S, O>(&self, stream: &mut S, output_file: &mut O) -> Result<(), Errors>
    where
        S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
        O: AsyncWrite + Unpin
    {
        let base_dirs = BaseDirs::new().unwrap();
        let mut path_buffer = PathBuf::new();

        path_buffer.push(base_dirs.cache_dir());
        path_buffer.push("rust-analyzer.gz");
        debug!("Temp file path: {}", path_buffer.display());

        let mut tmp_file = File::create(&path_buffer).await?;

        debug!("Copying Stream to Temp file");
        while let Some(chunk) = stream.next().await {
            let chunk_data: Bytes = chunk?;
            let mut cursor = Cursor::new(chunk_data);
            match tokio::io::copy(&mut cursor, &mut tmp_file).await {
                Ok(_) => {
                    debug!("Copied chunk to temp file {temp_file}", temp_file=path_buffer.display());
                }
                Err(e) => {
                    error!("Some error has occurred while copying stream to temp file: {} TempFile {temp_file}", e, temp_file=path_buffer.display());
                    drop(tmp_file);
                    tokio::fs::remove_file(&path_buffer).await?;
                    return Err(Errors::File(e));
                }
            }
        }
        debug!("Copying to TempFile finished");

        debug!("Starting decompression");
        drop(tmp_file);
        let mut gzip_decoder = GzipDecoder::new(BufReader::new(File::open(&path_buffer).await?));

        match tokio::io::copy(&mut gzip_decoder, output_file).await {
            Ok(_) => {
                debug!("Decompression finished, removing temp file");
                tokio::fs::remove_file(&path_buffer).await?;
                Ok(())
            }
            Err(err) => {
                error!(
                    "Some error has occurred while decompressing: {} TempFile {temp_file}",
                    err,
                    temp_file = path_buffer.display()
                );
                tokio::fs::remove_file(&path_buffer).await?;
                Err(Errors::File(err))
            }
        }
    }

    #[tracing::instrument]
    pub(super) async fn execute(self) -> Result<(), Errors> {
        let url = self.get_download_url();
        debug!("Downloading from: {url}", url = url);
        let res = self.client.get(url).send().await?;
        debug!("Response status: {status}", status = res.status());

        let mut stream = res.bytes_stream();
        let mut file = File::create(&self.output).await?;

        #[cfg(target_family="unix")]
        debug!("Setting permissions to file to 755 executable");
        #[cfg(target_family="unix")]
        file.set_permissions(Permissions::from_mode(0o755)).await?;

        match self.decompress(&mut stream, &mut file).await {
            Ok(_) => Ok(()),
            Err(e) => {
                drop(file);
                tokio::fs::remove_file(&self.output).await?;
                Err(e)
            }
        }
    }

    fn get_download_url(&self) -> String {
        format!(
            "https://github.com/rust-lang/rust-analyzer/releases/download/{}/{}",
            &self.version,
            self.get_file_name(),
        )
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
}
