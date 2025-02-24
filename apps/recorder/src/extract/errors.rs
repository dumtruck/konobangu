use std::{borrow::Cow, error::Error as StdError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Extract bangumi season error: {0}")]
    BangumiSeasonError(#[from] std::num::ParseIntError),
    #[error("Extract file url error: {0}")]
    FileUrlError(#[from] url::ParseError),
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

impl ExtractError {
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
