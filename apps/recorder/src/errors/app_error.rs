use std::borrow::Cow;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use fetch::{FetchError, HttpClientError, reqwest, reqwest_middleware};
use http::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use snafu::Snafu;

use crate::{
    auth::AuthError,
    crypto::CryptoError,
    downloader::DownloaderError,
    errors::{OptDynErr, response::StandardErrorResponse},
};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum RecorderError {
    #[snafu(transparent, context(false))]
    FancyRegexError {
        #[snafu(source(from(fancy_regex::Error, Box::new)))]
        source: Box<fancy_regex::Error>,
    },
    #[snafu(transparent)]
    NetAddrParseError { source: std::net::AddrParseError },
    #[snafu(transparent)]
    RegexError { source: regex::Error },
    #[snafu(transparent)]
    InvalidMethodError { source: http::method::InvalidMethod },
    #[snafu(transparent)]
    InvalidHeaderNameError {
        source: http::header::InvalidHeaderName,
    },
    #[snafu(transparent)]
    TracingAppenderInitError {
        source: tracing_appender::rolling::InitError,
    },
    #[snafu(transparent)]
    GraphQLSchemaError {
        source: async_graphql::dynamic::SchemaError,
    },
    #[snafu(transparent)]
    AuthError { source: AuthError },
    #[snafu(transparent)]
    DownloadError { source: DownloaderError },
    #[snafu(transparent)]
    RSSError { source: rss::Error },
    #[snafu(transparent)]
    DotEnvError { source: dotenvy::Error },
    #[snafu(transparent)]
    TeraError { source: tera::Error },
    #[snafu(transparent)]
    IOError { source: std::io::Error },
    #[snafu(transparent)]
    DbError { source: sea_orm::DbErr },
    #[snafu(transparent)]
    DbSqlxError { source: sea_orm::SqlxError },
    #[snafu(transparent, context(false))]
    FigmentError {
        #[snafu(source(from(figment::Error, Box::new)))]
        source: Box<figment::Error>,
    },
    #[snafu(transparent)]
    SerdeJsonError { source: serde_json::Error },
    #[snafu(transparent)]
    ParseUrlError { source: url::ParseError },
    #[snafu(display("{source}"), context(false))]
    OpenDALError {
        #[snafu(source(from(opendal::Error, Box::new)))]
        source: Box<opendal::Error>,
    },
    #[snafu(transparent)]
    InvalidHeaderValueError {
        source: http::header::InvalidHeaderValue,
    },
    #[snafu(transparent)]
    HttpClientError { source: HttpClientError },
    #[cfg(all(any(test, feature = "playground"), feature = "testcontainers"))]
    #[snafu(transparent)]
    TestcontainersError {
        source: testcontainers::TestcontainersError,
    },
    #[snafu(display("Extract {desc} with mime error, expected {expected}, but got {found}"))]
    MimeError {
        desc: String,
        expected: String,
        found: String,
    },
    #[snafu(display("Invalid or unknown format in extracting mikan rss"))]
    MikanRssInvalidFormatError,
    #[snafu(display("Invalid field {field} in extracting mikan rss"))]
    MikanRssInvalidFieldError {
        field: Cow<'static, str>,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
    #[snafu(display("Missing field {field} in extracting mikan meta"))]
    MikanMetaMissingFieldError {
        field: Cow<'static, str>,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
    #[snafu(display("Model Entity {entity} not found"))]
    ModelEntityNotFound { entity: Cow<'static, str> },
    #[snafu(transparent)]
    FetchError { source: FetchError },
    #[snafu(display("Credential3rdError: {message}, source = {source}"))]
    Credential3rdError {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
    #[snafu(transparent)]
    CryptoError { source: CryptoError },
    #[snafu(transparent)]
    StringFromUtf8Error { source: std::string::FromUtf8Error },
    #[snafu(display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
}

impl RecorderError {
    pub fn from_mikan_meta_missing_field(field: Cow<'static, str>) -> Self {
        Self::MikanMetaMissingFieldError {
            field,
            source: None.into(),
        }
    }

    pub fn from_mikan_rss_invalid_field(field: Cow<'static, str>) -> Self {
        Self::MikanRssInvalidFieldError {
            field,
            source: None.into(),
        }
    }

    pub fn from_mikan_rss_invalid_field_and_source(
        field: Cow<'static, str>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::MikanRssInvalidFieldError {
            field,
            source: OptDynErr::some_boxed(source),
        }
    }

    pub fn from_db_record_not_found<T: ToString>(detail: T) -> Self {
        Self::DbError {
            source: sea_orm::DbErr::RecordNotFound(detail.to_string()),
        }
    }
}

impl snafu::FromString for RecorderError {
    type Source = Box<dyn std::error::Error + Send + Sync>;

    fn without_source(message: String) -> Self {
        Self::Whatever {
            message,
            source: OptDynErr::none(),
        }
    }

    fn with_source(source: Self::Source, message: String) -> Self {
        Self::Whatever {
            message,
            source: OptDynErr::some(source),
        }
    }
}

impl IntoResponse for RecorderError {
    fn into_response(self) -> Response {
        match self {
            Self::AuthError { source: auth_error } => auth_error.into_response(),
            err => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json::<StandardErrorResponse>(StandardErrorResponse::from(err.to_string())),
            )
                .into_response(),
        }
    }
}

impl Serialize for RecorderError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RecorderError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::Whatever {
            message: s,
            source: None.into(),
        })
    }
}

impl From<reqwest::Error> for RecorderError {
    fn from(error: reqwest::Error) -> Self {
        FetchError::from(error).into()
    }
}

impl From<reqwest_middleware::Error> for RecorderError {
    fn from(error: reqwest_middleware::Error) -> Self {
        FetchError::from(error).into()
    }
}

pub type RecorderResult<T> = Result<T, RecorderError>;
