use reqwest::Url;

use crate::app::AppResult;

pub async fn fetch_content(url: &str) -> AppResult<Option<String>> {
    if Url::parse(url).is_err() {
        return Ok(None);
    }

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let content = response.text().await?;

    if content.is_empty() {
        return Ok(None);
    }

    Ok(Some(content))
}
