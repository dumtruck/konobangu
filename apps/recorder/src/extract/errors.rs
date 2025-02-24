use std::{borrow::Cow, error::Error as StdError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Parse bangumi season error: {0}")]
    BangumiSeasonError(#[from] std::num::ParseIntError),
    #[error("Parse file url error: {0}")]
    FileUrlError(#[from] url::ParseError),
    #[error("Parse {desc} with mime error, expected {expected}, but got {found}")]
    MimeError {
        desc: String,
        expected: String,
        found: String,
    },
    #[error("Parse mikan rss {url} format error")]
    MikanRssFormatError { url: String },
    #[error("Parse mikan rss item format error, {reason}")]
    MikanRssItemFormatError { reason: String },
    #[error("Missing field {field} in extracting meta")]
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
}
