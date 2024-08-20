use std::{env, error::Error, io::Cursor};

use fead::FormatText;
use fetch::get_content;
use html2text::from_read;
use htmlentity::entity::{decode, ICodedDataTrait};
use termimad::{minimad::TextTemplate, print_inline, MadSkin};

mod fetch;
mod reader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bind = env::args()
        .nth(1)
        .unwrap_or_else(|| "https://blog.rust-lang.org/feed.xml".to_string());
    let url = bind.as_str();

    let skin = MadSkin::default_dark();
    let (x, _) = termion::terminal_size().unwrap();

    let resp = get_content(url).await?;

    let data = reader::read_entries(resp.as_str())?;

    let first = data.first();

    match first {
        None => {
            print_inline("No entries found.\n");
            return Ok(());
        }
        Some(_) => (),
    }

    let raw_title = &first.unwrap().title.as_bytes().to_vec();
    let raw_content = &first.unwrap().content.as_bytes().to_vec();

    let decoded_title = decode(raw_title);
    let decoded_content = decode(raw_content);

    let title_text = decoded_title
        .to_chars()?
        .iter()
        .collect::<String>()
        .strip_trailing_newline();
    let content_text = decoded_content.to_chars()?.iter().collect::<String>();

    let title_cursor = Cursor::new(title_text);
    let content_cursor = Cursor::new(content_text);

    let title_binding = from_read(title_cursor, x.into());
    let title = title_binding.as_str();
    let content_binding = from_read(content_cursor, x.into());
    let content = content_binding.as_str();

    let text_template = TextTemplate::from(
        r#"
    # ${title}
    
    ${content}
    "#,
    );

    let mut expander = text_template.expander();

    expander
        .set("title", title)
        .set_lines_md("content", content);

    skin.print_expander(expander);

    Ok(())
}
