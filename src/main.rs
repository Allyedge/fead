use std::io;

use fead::app::{App, AppResult};
use fead::event::{Event, EventHandler};
use fead::handler::handle_key_events;
use fead::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut app = App::new();

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // let skin = MadSkin::default_dark();
    // let (x, _) = termion::terminal_size().unwrap();

    // let resp = get_content(url).await?;

    // let data = reader::read_entries(resp.as_str())?;

    // let first = data.get(index);

    // match first {
    //     None => {
    //         print_inline("No entries found.\n");
    //         return Ok(());
    //     }
    //     Some(_) => (),
    // }

    // let raw_title = &first.unwrap().title.as_bytes().to_vec();
    // let raw_content = &first.unwrap().content.as_bytes().to_vec();

    // let decoded_title = decode(raw_title);
    // let decoded_content = decode(raw_content);

    // let title_text = decoded_title
    //     .to_chars()?
    //     .iter()
    //     .collect::<String>()
    //     .strip_trailing_newline();
    // let content_text = decoded_content.to_chars()?.iter().collect::<String>();

    // let title_cursor = Cursor::new(title_text);
    // let content_cursor = Cursor::new(content_text);

    // let title_binding = from_read(title_cursor, x.into());
    // let title = title_binding.as_str();
    // let content_binding = from_read(content_cursor, x.into());
    // let content = content_binding.as_str();

    // let text_template = TextTemplate::from(
    //     r#"
    // # ${title}

    // ${content}
    // "#,
    // );

    // let mut expander = text_template.expander();

    // expander
    //     .set("title", title)
    //     .set_lines_md("content", content);

    // skin.print_expander(expander);

    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;

    Ok(())
}
