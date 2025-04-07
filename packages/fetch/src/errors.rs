use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FetchError {
    #[snafu(transparent)]
    CookieParseError { source: cookie::ParseError },
    #[snafu(transparent)]
    ReqwestError { source: reqwest::Error },
    #[snafu(transparent)]
    RequestMiddlewareError { source: reqwest_middleware::Error },
}
