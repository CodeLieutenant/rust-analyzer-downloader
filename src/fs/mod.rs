#[cfg(feature = "tokio")]
use tokio::io::{AsyncRead, AsyncWrite};

pub(crate) async fn copy<'a, R, W>(reader: &'a mut R, writer: &'a mut W) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    #[cfg(feature = "tokio")]
    return tokio::io::copy(reader, writer).await;
}
