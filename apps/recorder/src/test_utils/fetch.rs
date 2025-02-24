use color_eyre::eyre;

use crate::fetch::HttpClient;

pub fn build_testing_http_client() -> eyre::Result<HttpClient> {
    let mikan_client = HttpClient::default();
    Ok(mikan_client)
}
