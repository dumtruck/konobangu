use reqwest::IntoUrl;

use super::client::HttpClientTrait;
use crate::FetchError;

pub async fn fetch_html<T: IntoUrl, H: HttpClientTrait>(
    client: &H,
    url: T,
) -> Result<String, FetchError> {
    let content = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(content)
}
