use crate::{errors::app_error::RResult, fetch::HttpClient};

pub fn build_testing_http_client() -> RResult<HttpClient> {
    let mikan_client = HttpClient::default();
    Ok(mikan_client)
}
