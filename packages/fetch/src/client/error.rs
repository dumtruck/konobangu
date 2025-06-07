use axum::http;
use snafu::Snafu;
use util::OptDynErr;

#[derive(Debug, Snafu)]
pub enum HttpClientError {
    #[snafu(transparent)]
    ReqwestError { source: reqwest::Error },
    #[snafu(transparent)]
    ReqwestMiddlewareError { source: reqwest_middleware::Error },
    #[snafu(transparent)]
    HttpError { source: http::Error },
    #[snafu(display("Failed to parse cookies: {}", source))]
    ParseCookiesError { source: serde_json::Error },
    #[snafu(display("Failed to save cookies, message: {}, source: {:?}", message, source))]
    SaveCookiesError { message: String, source: OptDynErr },
    #[snafu(display("Failed to parse fetch client proxy: {source}"))]
    ProxyParseError { source: reqwest::Error },
    #[snafu(display("Failed to parse fetch client proxy auth header"))]
    ProxyAuthHeaderParseError,
}
