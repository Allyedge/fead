use crate::{
    app::{App, AppResult, InputMode},
    feeds::FeedManager,
    fetch::fetch_content,
    reader::read_title,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    // Exit application on `Ctrl-C`
    if (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
        && key_event.modifiers == KeyModifiers::CONTROL
    {
        app.quit();
    }

    match app.input_mode {
        InputMode::Normal => match key_event.code {
            KeyCode::Esc => app.input_mode = InputMode::Editing,
            KeyCode::Left => app.feed_list.state.select(None),
            KeyCode::Down => app.feed_list.state.select_next(),
            KeyCode::Up => app.feed_list.state.select_previous(),
            KeyCode::Home => app.feed_list.state.select_first(),
            KeyCode::End => app.feed_list.state.select_last(),
            KeyCode::Enter => match app.input_mode {
                InputMode::Normal => {
                    // enter
                }
                InputMode::Editing => {}
            },
            _ => {}
        },
        InputMode::Editing => {
            match key_event.code {
                KeyCode::Enter => {
                    app.feed_list.items.add_feed(
                        read_title(fetch_content(app.input.value()).await.unwrap().as_str())?,
                        app.input.value().to_string(),
                    );
                    app.feed_list.items.persist()?;
                    app.input.reset();
                    app.input_mode = InputMode::Normal;
                }
                KeyCode::Esc => app.input_mode = InputMode::Normal,
                _ => {}
            }

            app.input.handle_event(&Event::Key(key_event));
        }
    }
    Ok(())
}
