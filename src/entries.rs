use std::io::Cursor;

use html2text::from_read;
use htmlentity::entity::{decode, ICodedDataTrait};
use serde::{Deserialize, Serialize};

use crate::{app::AppResult, reader::read_entries, FormatText};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    pub description: String,
    pub content: String,
}

pub fn load_entries(xml: &str) -> AppResult<Vec<Entry>> {
    let entries = read_entries(xml)?;

    let mut result = vec![];

    for entry in entries {
        let raw_title = &entry.title.as_bytes().to_vec();
        let raw_description = &entry.description.as_bytes().to_vec();
        let raw_content = &entry.content.as_bytes().to_vec();

        let decoded_title = decode(raw_title);
        let decoded_description = decode(raw_description);
        let decoded_content = decode(raw_content);

        let title_text = decoded_title
            .to_chars()?
            .iter()
            .collect::<String>()
            .strip_trailing_newline();
        let description_text = decoded_description.to_chars()?.iter().collect::<String>();
        let content_text = decoded_content.to_chars()?.iter().collect::<String>();

        let title_cursor = Cursor::new(title_text);
        let description_cursor = Cursor::new(description_text);
        let content_cursor = Cursor::new(content_text);

        let title_binding = from_read(title_cursor, 80_usize);
        let title = title_binding.as_str();
        let description_binding = from_read(description_cursor, 80_usize);
        let description = description_binding.as_str();
        let content_binding = from_read(content_cursor, 80_usize);
        let content = content_binding.as_str();

        result.push(Entry {
            title: title.to_string(),
            description: description.to_string(),
            content: content.to_string(),
        });
    }

    Ok(result)
}
