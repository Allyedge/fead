use unicode_segmentation::UnicodeSegmentation;

use crate::feed::entries::{ContentKind, EntryContent};
use crate::feed::reader::html_to_plain_text;

const MAX_UNIT_CHARS: usize = 400;
const SILENCE_MS: u32 = 350;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NarrationUnit {
    Speech(String),
    Silence { ms: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NarrationTextError {
    NoNarratableContent,
}

pub fn build_narration_units(
    title: &str,
    body: &EntryContent,
) -> Result<Vec<NarrationUnit>, NarrationTextError> {
    let mut units = Vec::new();

    let title = normalize_whitespace(title);
    if !title.is_empty() {
        units.extend(segment_text(&title).into_iter().map(NarrationUnit::Speech));
        units.push(NarrationUnit::Silence { ms: SILENCE_MS });
    }

    let body_text = match body.kind {
        ContentKind::Text => clean_plain_text(&body.value),
        ContentKind::Html => clean_plain_text(&html_to_plain_text(&body.value)),
    };

    for piece in segment_text(&body_text) {
        units.push(NarrationUnit::Speech(piece));
    }

    units.retain(|unit| match unit {
        NarrationUnit::Speech(text) => !text.is_empty(),
        NarrationUnit::Silence { .. } => true,
    });

    if !units
        .iter()
        .any(|unit| matches!(unit, NarrationUnit::Speech(_)))
    {
        return Err(NarrationTextError::NoNarratableContent);
    }

    Ok(units)
}

fn clean_plain_text(text: &str) -> String {
    let without_urls = strip_bare_urls(text);
    normalize_whitespace(&without_urls)
}

fn strip_bare_urls(text: &str) -> String {
    text.split_whitespace()
        .filter(|token| {
            let lower = token.to_ascii_lowercase();
            !(lower.starts_with("http://")
                || lower.starts_with("https://")
                || lower.starts_with("www."))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn segment_text(text: &str) -> Vec<String> {
    let text = normalize_whitespace(text);
    if text.is_empty() {
        return Vec::new();
    }

    let mut units = Vec::new();
    for sentence in text.unicode_sentences() {
        let sentence = normalize_whitespace(sentence);
        if sentence.is_empty() {
            continue;
        }
        units.extend(split_long_unit(&sentence));
    }

    if units.is_empty() {
        units.extend(split_long_unit(&text));
    }

    units
}

fn split_long_unit(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= MAX_UNIT_CHARS {
        return vec![text.to_string()];
    }

    let mut units = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let remaining = chars.len() - start;
        if remaining <= MAX_UNIT_CHARS {
            let piece: String = chars[start..].iter().collect();
            let piece = normalize_whitespace(&piece);
            if !piece.is_empty() {
                units.push(piece);
            }
            break;
        }

        let hard_end = start + MAX_UNIT_CHARS;
        let window = &chars[start..hard_end];
        let split_at = find_split_offset(window).unwrap_or(MAX_UNIT_CHARS);
        let end = start + split_at.max(1);
        let piece: String = chars[start..end].iter().collect();
        let piece = normalize_whitespace(&piece);
        if !piece.is_empty() {
            units.push(piece);
        }
        start = end;
        while start < chars.len() && chars[start].is_whitespace() {
            start += 1;
        }
    }

    units
}

fn find_split_offset(window: &[char]) -> Option<usize> {
    let punct = window
        .iter()
        .rposition(|c| matches!(c, '.' | '!' | '?' | ';' | ':' | ',' | '…'));
    if let Some(idx) = punct {
        if idx + 1 >= window.len() / 4 {
            return Some(idx + 1);
        }
    }

    window
        .iter()
        .rposition(|c| c.is_whitespace())
        .filter(|&idx| idx + 1 >= window.len() / 4)
        .map(|idx| idx + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::entries::{ContentKind, EntryContent};

    #[test]
    fn builds_title_silence_and_body_units() {
        let body = EntryContent {
            value: "Hello world. Second sentence.".into(),
            kind: ContentKind::Text,
        };
        let units = build_narration_units("Title", &body).unwrap();
        assert!(matches!(&units[0], NarrationUnit::Speech(text) if text == "Title"));
        assert!(matches!(&units[1], NarrationUnit::Silence { ms: 350 }));
        assert!(matches!(&units[2], NarrationUnit::Speech(text) if text.contains("Hello")));
        assert!(units
            .iter()
            .any(|unit| matches!(unit, NarrationUnit::Speech(text) if text.contains("Second"))));
    }

    #[test]
    fn strips_urls_and_rejects_empty() {
        let body = EntryContent {
            value: "https://example.com www.example.org".into(),
            kind: ContentKind::Text,
        };
        assert_eq!(
            build_narration_units("", &body),
            Err(NarrationTextError::NoNarratableContent)
        );
    }

    #[test]
    fn splits_very_long_units() {
        let long = "word ".repeat(200);
        let body = EntryContent {
            value: long,
            kind: ContentKind::Text,
        };
        let units = build_narration_units("", &body).unwrap();
        assert!(units.len() > 1);
        for unit in units {
            if let NarrationUnit::Speech(text) = unit {
                assert!(text.chars().count() <= MAX_UNIT_CHARS);
            }
        }
    }

    #[test]
    fn converts_html_to_speech_text() {
        let body = EntryContent {
            value: "<p>Hello <a href=\"https://x.test\">there</a>.</p><script>bad()</script>"
                .into(),
            kind: ContentKind::Html,
        };
        let units = build_narration_units("News", &body).unwrap();
        let speech: Vec<_> = units
            .into_iter()
            .filter_map(|unit| match unit {
                NarrationUnit::Speech(text) => Some(text),
                NarrationUnit::Silence { .. } => None,
            })
            .collect();
        let joined = speech.join(" ");
        assert!(joined.contains("Hello"));
        assert!(joined.contains("there"));
        assert!(!joined.contains("bad()"));
        assert!(!joined.contains("https://"));
    }
}
