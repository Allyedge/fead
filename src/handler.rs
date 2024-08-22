use crate::app::{App, AppResult, InputMode};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    // Exit application on `Ctrl-C`
    if (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
        && key_event.modifiers == KeyModifiers::CONTROL
    {
        app.quit();
    }

    match app.input_mode {
        InputMode::Normal => match key_event.code {
            KeyCode::Esc => match app.input_mode {
                InputMode::Normal => app.input_mode = InputMode::Editing,
                InputMode::Editing => {}
            },
            KeyCode::Enter => match app.input_mode {
                InputMode::Normal => {
                    // enter the selected feed
                }
                InputMode::Editing => {}
            },
            _ => {}
        },
        InputMode::Editing => {
            match key_event.code {
                KeyCode::Enter => {
                    // submit the feed url TODO
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