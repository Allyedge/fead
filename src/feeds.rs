use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::app::AppResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feed {
    pub title: String,
    pub url: String,
}

pub trait FeedsManager {
    fn persist(&self) -> AppResult<()>;
    fn add_feed(&mut self, title: String, url: String);
    fn remove_feed(&mut self, title: &str);
}

impl FeedsManager for Vec<Feed> {
    fn persist(&self) -> AppResult<()> {
        let feeds_json = serde_json::to_string_pretty(self)?;
        fs::write("feeds.json", format!("{feeds_json}\n"))?;
        Ok(())
    }

    fn add_feed(&mut self, title: String, url: String) {
        self.push(Feed { title, url });
    }

    fn remove_feed(&mut self, title: &str) {
        self.retain(|feed| feed.title != title);
    }
}

pub fn load_feeds() -> AppResult<Vec<Feed>> {
    let path = Path::new("feeds.json");

    if !path.exists() {
        fs::write(path, "[]\n")?;
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)?;
    if contents.trim().is_empty() {
        fs::write(path, "[]\n")?;
        return Ok(Vec::new());
    }

    Ok(serde_json::from_str(&contents)?)
}
