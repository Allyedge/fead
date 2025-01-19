use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use std::error::Error;

use crate::entries::Entry;

fn read_generator(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<String, Box<dyn Error>> {
    loop {
        match reader.read_event_into(buf)? {
            Event::Start(element) => {
                if element.name().as_ref() == b"generator" {
                    return Ok(reader.read_text(QName(b"generator"))?.to_string());
                }
            }
            Event::End(element) => {
                if element.name().as_ref() == b"generator" {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(String::new())
}

fn read_channel_title(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> Result<String, Box<dyn Error>> {
    loop {
        match reader.read_event_into(buf)? {
            Event::Start(element) => {
                if element.name().as_ref() == b"title" {
                    return Ok(reader.read_text(QName(b"title"))?.to_string());
                }
            }
            Event::End(element) => {
                if element.name().as_ref() == b"title" {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(String::new())
}

fn read_entry(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<Entry, Box<dyn Error>> {
    let mut entry = Entry {
        title: String::new(),
        description: String::new(),
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
                    let description = reader.read_text(QName(b"description"))?;
                    entry.description.push_str(&description);
                }
                QName(b"content") => {
                    let content = reader.read_text(QName(b"content"))?;
                    entry.content.push_str(&content);
                }
                QName(b"content:encoded") => {
                    let content = reader.read_text(QName(b"content:encoded"))?;
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

fn read_item(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<Entry, Box<dyn Error>> {
    let mut entry = Entry {
        title: String::new(),
        description: String::new(),
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
                    let description = reader.read_text(QName(b"description"))?;
                    entry.description.push_str(&description);
                }
                QName(b"content") => {
                    let content = reader.read_text(QName(b"content"))?;
                    entry.content.push_str(&content);
                }
                QName(b"content:encoded") => {
                    let content = reader.read_text(QName(b"content:encoded"))?;
                    entry.content.push_str(&content);
                }
                QName(b"summary") => {
                    let summary = reader.read_text(QName(b"summary"))?;
                    entry.description.push_str(&summary);
                }
                QName(b"media:description") => {
                    let media_description = reader.read_text(QName(b"media:description"))?;
                    entry.description.push_str(&media_description);
                }
                QName(b"media:content") => {
                    let media_content = reader.read_text(QName(b"media:content"))?;
                    entry.content.push_str(&media_content);
                }
                QName(b"media:title") => {
                    let media_title = reader.read_text(QName(b"media:title"))?;
                    entry.title.push_str(&media_title);
                }
                _ => (),
            },
            Event::End(element) => {
                if element.name().as_ref() == b"item" || element.name().as_ref() == b"entry" {
                    break;
                }
            }
            Event::Eof => break,
            _ => (),
        }
        buf.clear();
    }

    Ok(entry)
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
                QName(b"item") | QName(b"entry") => {
                    let mut entry = read_item(&mut reader, &mut buf)?;
                    entry.content = clean_content(entry.content);
                    entries.push(entry);
                }
                QName(b"channel") | QName(b"feed") => {
                    let mut channel_entries = read_channel(&mut reader, &mut buf)?;
                    for entry in &mut channel_entries {
                        entry.content = clean_content(entry.content.clone());
                    }
                    entries.extend(channel_entries);
                }
                _ => (),
            },
            _ => (),
        }
        buf.clear();
    }

    Ok(entries)
}

pub fn read_title(xml: &str) -> Result<String, Box<dyn Error>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(e.into()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"channel" {
                    return read_channel_title(&mut reader, &mut buf);
                }

                if e.name().as_ref() == b"feed" {
                    return read_generator(&mut reader, &mut buf);
                }
            }
            _ => (),
        }
    }

    buf.clear();

    Ok(String::new())
}

fn clean_content(content: String) -> String {
    content.replace("]]>", "").replace("<![CDATA[", "")
}
