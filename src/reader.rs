use std::error::Error;

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Entry {
    pub title: String,
    pub content: String,
}

pub fn read_entries(xml: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut reader = Reader::from_str(xml);

    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut entries = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(e.into()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                if let QName(b"entry") = e.name() {
                    let mut entry = Entry {
                        title: String::new(),
                        content: String::new(),
                    };

                    loop {
                        match reader.read_event_into(&mut buf)? {
                            Event::Start(element) => match element.name() {
                                QName(b"title") => {
                                    let title = reader.read_text(QName(b"title"))?;
                                    entry.title.push_str(&title);
                                }
                                QName(b"content") => {
                                    let content = reader.read_text(QName(b"content"))?;
                                    entry.content.push_str(&content);
                                }
                                _ => (),
                            },
                            Event::End(element) => {
                                if element.name().as_ref() == b"entry" {
                                    entries.push(entry);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => (),
        }
    }

    buf.clear();

    Ok(entries)
}
