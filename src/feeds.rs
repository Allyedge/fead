use std::{fs::File, io::Write, path::Path};

use serde::{Deserialize, Serialize};

use crate::app::AppResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    pub title: String,
    pub url: String,
}

pub trait FeedManager {
    fn persist(&self) -> AppResult<()>;
    fn add_feed(&mut self, title: String, url: String);
    fn remove_feed(&mut self, title: &str);
}

impl FeedManager for Vec<Feed> {
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

pub fn load() -> AppResult<Vec<Feed>> {
    let exists = Path::exists(Path::new("feeds.json"));

    match exists {
        true => {
            let file = File::open(Path::new("feeds.json"))?;
            let feeds: Vec<Feed> = serde_json::from_reader(file)?;
            Ok(feeds)
        }
        false => {
            let _ = File::create(Path::new("feeds.json"))?;
            let feeds: Vec<Feed> = vec![];
            Ok(feeds)
        }
    }
}
