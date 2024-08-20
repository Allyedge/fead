use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use std::error::Error;

#[derive(Debug)]
pub struct Entry {
    pub title: String,
    pub content: String,
}

fn read_entry(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<Entry, Box<dyn Error>> {
    let mut entry = Entry {
        title: String::new(),
        content: String::new(),
    };

    loop {
        match reader.read_event_into(buf)? {
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
                    return Ok(entry);
                }
            }
            _ => {}
        }
    }
}

fn read_channel(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut entries = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(element) => {
                if let QName(b"item") = element.name() {
                    let entry = read_item(reader, buf)?;
                    entries.push(entry);
                }
            }
            Event::End(element) => {
                if element.name().as_ref() == b"channel" {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(entries)
}

fn read_item(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<Entry, Box<dyn Error>> {
    let mut entry = Entry {
        title: String::new(),
        content: String::new(),
    };

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(element) => match element.name() {
                QName(b"title") => {
                    let title = reader.read_text(QName(b"title"))?;
                    entry.title.push_str(&title);
                }
                QName(b"description") => {
                    let content = reader.read_text(QName(b"description"))?;
                    entry.content.push_str(&content);
                }
                _ => (),
            },
            Event::End(element) => {
                if element.name().as_ref() == b"item" {
                    return Ok(entry);
                }
            }
            _ => {}
        }
    }
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
            Ok(Event::Start(e)) => match e.name() {
                QName(b"entry") => {
                    let entry = read_entry(&mut reader, &mut buf)?;
                    entries.push(entry);
                }
                QName(b"channel") => {
                    let channel_entries = read_channel(&mut reader, &mut buf)?;
                    entries.extend(channel_entries);
                }
                _ => (),
            },
            _ => (),
        }
    }

    buf.clear();

    Ok(entries)
}
