use reqwest::IntoUrl;

use super::client::HttpClientTrait;

pub async fn fetch_html<T: IntoUrl, H: HttpClientTrait>(
    client: &H,
    url: T,
) -> color_eyre::eyre::Result<String> {
    let content = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(content)
}
