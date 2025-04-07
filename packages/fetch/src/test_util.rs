use crate::{FetchError, HttpClient};

pub fn build_testing_http_client() -> Result<HttpClient, FetchError> {
    let mikan_client = HttpClient::default();
    Ok(mikan_client)
}
