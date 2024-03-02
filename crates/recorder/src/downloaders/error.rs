use std::borrow::Cow;
use thiserror::Error;
use std::time::Duration;

#[derive(Error, Debug)]
pub enum DownloaderError {
    #[error("Invalid mime (expected {expected:?}, got {found:?})")]
    InvalidMime { expected: String, found: String },
    #[error("Invalid url schema (expected {expected:?}, got {found:?})")]
    InvalidUrlSchema { expected: String, found: String },
    #[error("Invalid url parse: {0:?}")]
    InvalidUrlParse(#[from] url::ParseError),
    #[error("Invalid url format: {reason}")]
    InvalidUrlFormat { reason: Cow<'static, str> },
    #[error("QBit api error: {0:?}")]
    QBitAPIError(#[from] qbit_rs::Error),
    #[error("Timeout error ({action} timeouts out of {timeout:?})")]
    TimeoutError {
        action: Cow<'static, str>,
        timeout: Duration,
    },
    #[error("Invalid torrent file format")]
    InvalidTorrentFileFormat,
    #[error("Invalid magnet file format (url = {url})")]
    InvalidMagnetFormat { url: String },
}
