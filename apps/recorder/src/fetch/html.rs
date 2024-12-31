use reqwest::IntoUrl;

use super::HttpClient;

pub async fn fetch_html<T: IntoUrl>(client: Option<&HttpClient>, url: T) -> eyre::Result<String> {
    let client = client.unwrap_or_default();
    let content = client.get(url).send().await?.text().await?;

    Ok(content)
}
