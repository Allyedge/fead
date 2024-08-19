use std::error::Error;

pub async fn get_content(url: &str) -> Result<String, Box<dyn Error>> {
    let resp = reqwest::get(url).await?;

    let content = resp.text().await?;

    Ok(content)
}
