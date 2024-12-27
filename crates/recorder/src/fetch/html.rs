use reqwest::IntoUrl;

use super::{core::DEFAULT_HTTP_CLIENT_USER_AGENT, HttpClient};

pub async fn download_html<U: IntoUrl>(url: U) -> eyre::Result<String> {
    let request_client = reqwest::Client::builder()
        .user_agent(DEFAULT_HTTP_CLIENT_USER_AGENT)
        .build()?;
    let content = request_client.get(url).send().await?.text().await?;
    Ok(content)
}

pub async fn download_html_with_client<T: IntoUrl>(
    client: Option<&HttpClient>,
    url: T,
) -> eyre::Result<String> {
    if let Some(client) = client {
        let content = client.get(url).send().await?.text().await?;
        Ok(content)
    } else {
        download_html(url).await
    }
}
