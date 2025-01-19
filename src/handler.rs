use crate::{
    app::{App, AppResult, InputMode},
    entries::load_entries,
    feeds::FeedsManager,
    fetch::fetch_content,
    reader::read_title,
    screen::Screen,
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
            KeyCode::Esc => match app.screen {
                Screen::Home => app.input_mode = InputMode::Editing,
                Screen::Feed => app.screen = Screen::Home,
                Screen::Article => {
                    app.scroll_offset = 0;
                    app.screen = Screen::Feed
                }
            },
            KeyCode::Left => match app.screen {
                Screen::Home => {}
                Screen::Feed => app.screen = Screen::Home,
                Screen::Article => {
                    app.scroll_offset = 0;
                    app.screen = Screen::Feed;
                }
            },
            KeyCode::Down => match app.screen {
                Screen::Home => app.feed_list.state.select_next(),
                Screen::Feed => app.entry_list.state.select_next(),
                Screen::Article => {
                    let lines = app.current_entry.content.lines().count() as u16;
                    if app.scroll_offset < lines.saturating_sub(1) {
                        app.scroll_offset += 1;
                    }
                }
            },
            KeyCode::Up => match app.screen {
                Screen::Home => app.feed_list.state.select_previous(),
                Screen::Feed => app.entry_list.state.select_previous(),
                Screen::Article => {
                    if app.scroll_offset > 0 {
                        app.scroll_offset -= 1;
                    }
                }
            },
            KeyCode::Home => match app.screen {
                Screen::Home => app.feed_list.state.select_first(),
                Screen::Feed => app.entry_list.state.select_first(),
                Screen::Article => {}
            },
            KeyCode::End => match app.screen {
                Screen::Home => app.feed_list.state.select_last(),
                Screen::Feed => app.entry_list.state.select_last(),
                Screen::Article => {}
            },
            KeyCode::Enter => match app.input_mode {
                InputMode::Normal => match app.screen {
                    Screen::Home => {
                        if let Some(selected) = app.feed_list.state.selected() {
                            let data =
                                fetch_content(&app.feed_list.items.get(selected).unwrap().url)
                                    .await?;
                            app.entry_list.items = load_entries(data.as_str())?;
                            app.entry_list.state.select_first();
                            app.screen = Screen::Feed;
                        }
                    }
                    Screen::Feed => {
                        if let Some(selected) = app.entry_list.state.selected() {
                            let entry = app.entry_list.items.get(selected).unwrap().clone();
                            app.current_entry = entry;
                            app.screen = Screen::Article;
                        }
                    }
                    Screen::Article => {}
                },
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
