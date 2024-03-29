use reqwest::IntoUrl;

use super::defs::DEFAULT_USER_AGENT;

pub async fn download_html<U: IntoUrl>(url: U) -> eyre::Result<String> {
    let request_client = reqwest::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .build()?;
    let content = request_client.get(url).send().await?.text().await?;
    Ok(content)
}
