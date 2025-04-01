use std::{borrow::Cow, time::Duration};

use snafu::prelude::*;

use crate::errors::OptionWhateverAsync;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum DownloaderError {
    #[snafu(display("Invalid mime (expected {expected:?}, got {found:?})"))]
    DownloadMimeError { expected: String, found: String },
    #[snafu(display("Invalid url schema (expected {expected:?}, got {found:?})"))]
    DownloadSchemaError { expected: String, found: String },
    #[snafu(transparent)]
    DownloadUrlParseError { source: url::ParseError },
    #[snafu(display("Invalid url format: {reason}"))]
    DownloadUrlFormatError { reason: Cow<'static, str> },
    #[snafu(transparent)]
    QBitAPIError { source: qbit_rs::Error },
    #[snafu(display("Timeout error (action = {action}, timeout = {timeout:?})"))]
    DownloadTimeoutError {
        action: Cow<'static, str>,
        timeout: Duration,
    },
    #[snafu(display("Invalid torrent file format"))]
    TorrentFileFormatError,
    #[snafu(display("Invalid magnet format (url = {url})"))]
    MagnetFormatError { url: String },
    #[snafu(display("Failed to fetch: {source}"))]
    DownloadFetchError {
        #[snafu(source)]
        source: Box<dyn snafu::Error + Send + Sync>,
    },
    #[snafu(display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptionWhateverAsync::some)))]
        source: OptionWhateverAsync,
    },
}

impl snafu::FromString for DownloaderError {
    type Source = Box<dyn std::error::Error + Send + Sync>;

    fn without_source(message: String) -> Self {
        Self::Whatever {
            message,
            source: OptionWhateverAsync::none(),
        }
    }

    fn with_source(source: Self::Source, message: String) -> Self {
        Self::Whatever {
            message,
            source: OptionWhateverAsync::some(source),
        }
    }
}
