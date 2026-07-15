use feed_rs::{
    model::{Content, Entry as ParsedEntry, Link, Text},
    parser::{self, ParseFeedError},
};
use markup5ever_rcdom::NodeData;

use crate::entries::{ContentKind, Entry, EntryContent, FeedDocument};

pub fn parse_feed(source: &[u8]) -> Result<FeedDocument, ParseFeedError> {
    parser::parse(source).map(normalize_feed)
}

fn normalize_feed(feed: feed_rs::model::Feed) -> FeedDocument {
    FeedDocument {
        title: feed.title.map(normalize_title).unwrap_or_default(),
        entries: feed.entries.into_iter().map(normalize_entry).collect(),
    }
}

fn normalize_entry(entry: ParsedEntry) -> Entry {
    let published = entry
        .published
        .or(entry.updated)
        .map(|date| date.to_rfc3339());
    let title = entry
        .title
        .map(normalize_title)
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| "Untitled article".to_string());

    Entry {
        id: (!entry.id.is_empty()).then_some(entry.id),
        title,
        link: select_link(&entry.links),
        summary: entry.summary.and_then(normalize_text),
        content: entry.content.and_then(normalize_content),
        published,
    }
}

fn normalize_text(text: Text) -> Option<EntryContent> {
    let kind = content_kind(text.content_type.as_ref())?;
    non_empty_content(text.content, kind)
}

fn normalize_content(content: Content) -> Option<EntryContent> {
    let kind = content_kind(content.content_type.as_ref())?;
    non_empty_content(content.body?, kind)
}

fn non_empty_content(value: String, kind: ContentKind) -> Option<EntryContent> {
    (!value.trim().is_empty()).then_some(EntryContent { value, kind })
}

fn content_kind(media_type: &str) -> Option<ContentKind> {
    let essence = media_type
        .split_once(';')
        .map_or(media_type, |(essence, _)| essence)
        .trim()
        .to_ascii_lowercase();
    match essence.as_str() {
        "text/html" => Some(ContentKind::Html),
        "application/xhtml+xml" => Some(ContentKind::Html),
        essence if essence.starts_with("text/") => Some(ContentKind::Text),
        _ => None,
    }
}

fn select_link(links: &[Link]) -> Option<String> {
    links
        .iter()
        .find(|link| link.rel.as_deref().is_none_or(|rel| rel == "alternate"))
        .or_else(|| links.first())
        .map(|link| link.href.clone())
        .filter(|href| !href.trim().is_empty())
}

fn normalize_title(title: Text) -> String {
    let content = if content_kind(title.content_type.as_ref()) == Some(ContentKind::Html) {
        html_to_plain_text(&title.content)
    } else {
        title.content
    };
    content.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn html_to_plain_text(html: &str) -> String {
    let converter = htmd::HtmlToMarkdown::new();
    let Ok(document) = converter.html_to_tree(html) else {
        return html.to_string();
    };
    let mut output = String::new();
    collect_text(&document, &mut output);
    output
}

fn collect_text(node: &htmd::Node, output: &mut String) {
    match &node.data {
        NodeData::Text { contents } => output.push_str(&contents.borrow()),
        NodeData::Element { name, .. }
            if matches!(name.local.as_ref(), "script" | "style" | "template") =>
        {
            return;
        }
        _ => {}
    }

    for child in node.children.borrow().iter() {
        collect_text(child, output);
    }
}

#[cfg(test)]
mod tests {
    use crate::entries::ContentKind;

    use super::parse_feed;

    #[test]
    fn normalizes_rss_content_and_metadata() {
        let xml = r#"
            <rss version="2.0"
                 xmlns:content="http://purl.org/rss/1.0/modules/content/"
                 xmlns:media="http://search.yahoo.com/mrss/">
              <channel>
                <title>Example &amp; Company</title>
                <description>Example feed</description>
                <link>https://example.com</link>
                <item>
                  <guid>article-42</guid>
                  <title> A useful article </title>
                  <link>https://example.com/articles/42</link>
                  <pubDate>Tue, 15 Jul 2025 10:30:00 GMT</pubDate>
                  <description><![CDATA[<p>Short summary</p>]]></description>
                  <media:content url="https://example.com/video.mp4" type="video/mp4" />
                  <content:encoded><![CDATA[<article><p>Full story</p></article>]]></content:encoded>
                </item>
              </channel>
            </rss>
        "#;

        let feed = parse_feed(xml.as_bytes()).expect("valid RSS feed");
        let entry = &feed.entries[0];

        assert_eq!(feed.title, "Example & Company");
        assert_eq!(entry.id.as_deref(), Some("article-42"));
        assert_eq!(entry.title, "A useful article");
        assert_eq!(
            entry.link.as_deref(),
            Some("https://example.com/articles/42")
        );
        assert_eq!(
            entry.summary.as_ref().unwrap().value,
            "<p>Short summary</p>"
        );
        assert_eq!(entry.body().unwrap().kind, ContentKind::Html);
        assert_eq!(
            entry.body().unwrap().value,
            "<article><p>Full story</p></article>"
        );
    }

    #[test]
    fn normalizes_atom_links_dates_and_xhtml() {
        let xml = r#"
            <feed xmlns="http://www.w3.org/2005/Atom">
              <id>tag:example.com,2025:feed</id>
              <title type="html">Example &lt;strong&gt;Atom&lt;/strong&gt;</title>
              <updated>2025-07-15T09:00:00Z</updated>
              <entry>
                <id>tag:example.com,2025:7</id>
                <title type="html">Atom &lt;em&gt;entry&lt;/em&gt;</title>
                <updated>2025-07-15T09:00:00Z</updated>
                <published>2025-07-14T18:20:00Z</published>
                <link rel="self" href="https://example.com/api/7" />
                <link rel="alternate" href="https://example.com/articles/7" />
                <summary type="html">&lt;p&gt;Summary&lt;/p&gt;</summary>
                <content type="xhtml">
                  <div xmlns="http://www.w3.org/1999/xhtml"><p>Full &amp; proper</p></div>
                </content>
              </entry>
            </feed>
        "#;

        let feed = parse_feed(xml.as_bytes()).expect("valid Atom feed");
        let entry = &feed.entries[0];

        assert_eq!(feed.title, "Example Atom");
        assert_eq!(entry.title, "Atom entry");
        assert_eq!(entry.id.as_deref(), Some("tag:example.com,2025:7"));
        assert_eq!(
            entry.link.as_deref(),
            Some("https://example.com/articles/7")
        );
        assert_eq!(
            entry.published.as_deref(),
            Some("2025-07-14T18:20:00+00:00")
        );
        assert_eq!(entry.summary.as_ref().unwrap().value, "<p>Summary</p>");
        assert_eq!(entry.content.as_ref().unwrap().kind, ContentKind::Html);
        assert!(entry
            .content
            .as_ref()
            .unwrap()
            .value
            .contains("Full & proper"));
    }

    #[test]
    fn recognizes_parameterized_html_content() {
        let xml = r#"
            <feed xmlns="http://www.w3.org/2005/Atom">
              <id>tag:example.com,2025:feed</id>
              <title>Example</title>
              <updated>2025-07-15T09:00:00Z</updated>
              <entry>
                <id>tag:example.com,2025:8</id>
                <title>Parameterized HTML</title>
                <updated>2025-07-15T09:00:00Z</updated>
                <content type="text/html; charset=utf-8">&lt;p&gt;Story&lt;/p&gt;</content>
              </entry>
            </feed>
        "#;

        let feed = parse_feed(xml.as_bytes()).expect("valid Atom feed");
        assert_eq!(feed.entries[0].body().unwrap().kind, ContentKind::Html);
    }

    #[test]
    fn supports_rss1_and_rejects_truncated_xml() {
        let rss1 = r#"
            <rdf:RDF
                xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                xmlns="http://purl.org/rss/1.0/">
              <channel rdf:about="https://example.com/">
                <title>Example RSS 1</title>
                <link>https://example.com/</link>
                <description>Example feed</description>
              </channel>
              <item rdf:about="https://example.com/first">
                <title>First item</title>
                <link>https://example.com/first</link>
                <description>Only a summary</description>
              </item>
            </rdf:RDF>
        "#;

        let feed = parse_feed(rss1.as_bytes()).expect("valid RSS 1 feed");
        assert_eq!(feed.title, "Example RSS 1");
        assert_eq!(feed.entries[0].title, "First item");
        assert_eq!(feed.entries[0].body(), feed.entries[0].summary.as_ref());

        let truncated = b"<rss><channel><title>Broken</title><item>";
        assert!(parse_feed(truncated).is_err());
    }
}
