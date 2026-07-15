use std::{error::Error, fmt, time::Duration};

use reqwest::{Client, StatusCode, Url};

#[derive(Debug)]
pub enum FetchError {
    InvalidUrl,
    UnsupportedScheme,
    Request(reqwest::Error),
    HttpStatus(StatusCode),
    EmptyResponse,
}

impl fmt::Display for FetchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl => formatter.write_str("invalid feed URL"),
            Self::UnsupportedScheme => formatter.write_str("feed URL must use HTTP or HTTPS"),
            Self::Request(_) => formatter.write_str("feed request failed"),
            Self::HttpStatus(status) => write!(formatter, "feed returned HTTP {status}"),
            Self::EmptyResponse => formatter.write_str("feed returned an empty response"),
        }
    }
}

impl Error for FetchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(error) => Some(error),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

pub async fn fetch_content(url: &str) -> Result<Vec<u8>, FetchError> {
    let url = Url::parse(url).map_err(|_| FetchError::InvalidUrl)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(FetchError::UnsupportedScheme);
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent(concat!("fead/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(FetchError::HttpStatus(response.status()));
    }

    let content = response.bytes().await?;
    if content.is_empty() {
        return Err(FetchError::EmptyResponse);
    }

    Ok(content.to_vec())
}
