use reqwest::IntoUrl;

use super::client::HttpClientTrait;
use crate::errors::RecorderError;

pub async fn fetch_html<T: IntoUrl, H: HttpClientTrait>(
    client: &H,
    url: T,
) -> Result<String, RecorderError> {
    let content = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(content)
}
