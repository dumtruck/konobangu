use std::{borrow::Cow, error::Error as StdError};

use axum::response::{IntoResponse, Response};
use http::StatusCode;
use thiserror::Error as ThisError;

use crate::{auth::AuthError, fetch::HttpClientError};

#[derive(ThisError, Debug)]
pub enum RError {
    #[error(transparent)]
    InvalidMethodError(#[from] http::method::InvalidMethod),
    #[error(transparent)]
    InvalidHeaderNameError(#[from] http::header::InvalidHeaderName),
    #[error(transparent)]
    TracingAppenderInitError(#[from] tracing_appender::rolling::InitError),
    #[error(transparent)]
    GraphQLSchemaError(#[from] async_graphql::dynamic::SchemaError),
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error(transparent)]
    RSSError(#[from] rss::Error),
    #[error(transparent)]
    DotEnvError(#[from] dotenv::Error),
    #[error(transparent)]
    TeraError(#[from] tera::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    DbError(#[from] sea_orm::DbErr),
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
    #[error("Model Entity {entity} not found")]
    ModelEntityNotFound { entity: Cow<'static, str> },
    #[error("{0}")]
    CustomMessageStr(&'static str),
    #[error("{0}")]
    CustomMessageString(String),
}

impl RError {
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

    pub fn from_db_record_not_found<T: ToString>(detail: T) -> Self {
        Self::DbError(sea_orm::DbErr::RecordNotFound(detail.to_string()))
    }
}

impl IntoResponse for RError {
    fn into_response(self) -> Response {
        match self {
            Self::AuthError(auth_error) => auth_error.into_response(),
            err => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}

pub type RResult<T> = Result<T, RError>;
