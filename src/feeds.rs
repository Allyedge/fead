use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::app::AppResult;

#[derive(Debug, Serialize, Deserialize)]
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
        let mut file = File::create("feeds.json")?;
        let feeds_json = serde_json::to_string(self)?;
        file.write_all(feeds_json.as_bytes())?;
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
    let exists = Path::exists(path);

    match exists {
        true => {
            let file = File::open(path)?;

            let metadata = file.metadata()?;

            if metadata.is_file() && metadata.len() == 0 {
                fs::write(path, "[]").unwrap();
            }

            let feeds: Vec<Feed> = serde_json::from_reader(file)?;
            Ok(feeds)
        }
        false => {
            let _ = File::create(path)?;
            let feeds: Vec<Feed> = vec![];
            Ok(feeds)
        }
    }
}
