use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloaderError {
    #[error("Invalid mime (expected {expected:?}, got {found:?})")]
    InvalidMime { expected: String, found: String },
    #[error("Invalid url format")]
    InvalidUrlFormat(#[from] url::ParseError),
    #[error("QBit api error: {0:?}")]
    QBitAPIError(#[from] qbit_rs::Error),
}
