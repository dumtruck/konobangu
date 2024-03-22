use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
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
    #[error("Parse language tag error: {0}")]
    LanguageTagError(#[from] oxilangtag::LanguageTagParseError),
    #[error("Unsupported language preset: {0}")]
    UnsupportedLanguagePreset(String),
}
