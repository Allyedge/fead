use crate::app::{App, AppResult, InputMode};
use crossterm::event::{Event, KeyCode, KeyEvent};
use tui_input::backend::crossterm::EventHandler;

use super::feed_actions::add_feed;

pub(super) async fn handle_editing_mode(key: KeyEvent, app: &mut App) -> AppResult<()> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.notice = None;
        }
        KeyCode::Enter => add_feed(app).await?,
        _ => {
            app.input.handle_event(&Event::Key(key));
            app.notice = None;
        }
    }
    Ok(())
}
