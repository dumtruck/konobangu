use std::{borrow::Cow, error::Error as StdError};

use thiserror::Error as ThisError;

use crate::fetch::HttpClientError;

#[derive(ThisError, Debug)]
pub enum RecorderError {
    #[error(transparent)]
    CookieParseError(#[from] cookie::ParseError),
    #[error(transparent)]
    FigmentError(#[from] figment::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    ParseUrlError(#[from] url::ParseError),
    #[error(transparent)]
    OpenDALError(#[from] opendal::Error),
    #[error(transparent)]
    InvalidHeaderValueError(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    HttpClientError(#[from] HttpClientError),
    #[error("Extract {desc} with mime error, expected {expected}, but got {found}")]
    MimeError {
        desc: String,
        expected: String,
        found: String,
    },
    #[error("Invalid or unknown format in extracting mikan rss")]
    MikanRssInvalidFormatError,
    #[error("Invalid field {field} in extracting mikan rss")]
    MikanRssInvalidFieldError {
        field: Cow<'static, str>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("Missing field {field} in extracting mikan meta")]
    MikanMetaMissingFieldError {
        field: Cow<'static, str>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

impl RecorderError {
    pub fn from_mikan_meta_missing_field(field: Cow<'static, str>) -> Self {
        Self::MikanMetaMissingFieldError {
            field,
            source: None,
        }
    }

    pub fn from_mikan_rss_invalid_field(field: Cow<'static, str>) -> Self {
        Self::MikanRssInvalidFieldError {
            field,
            source: None,
        }
    }

    pub fn from_mikan_rss_invalid_field_and_source(
        field: Cow<'static, str>,
        source: Box<dyn StdError + Send + Sync>,
    ) -> Self {
        Self::MikanRssInvalidFieldError {
            field,
            source: Some(source),
        }
    }
}

impl From<RecorderError> for loco_rs::Error {
    fn from(error: RecorderError) -> Self {
        Self::wrap(error)
    }
}
