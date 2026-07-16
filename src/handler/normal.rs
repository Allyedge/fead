use crate::app::{
    App, AppResult, ConfirmationChoice, ConfirmationKind, ConfirmationPopup, InputMode,
};
use crate::screen::Screen;
use crate::tts::NarrationHandle;
use crossterm::event::{KeyCode, KeyEvent};

use super::feed_actions::open_selection;
use super::navigation::{go_back, move_selection, select_edge, Direction, Edge};
use super::tts::{request_tts, stop_narration, toggle_narration};

pub(super) async fn handle_normal_mode(
    key: KeyEvent,
    app: &mut App,
    narration: &NarrationHandle,
) -> AppResult<()> {
    match key.code {
        KeyCode::Char('q') => {
            stop_narration(app, narration);
            app.quit();
        }
        KeyCode::Char('a' | '/') if app.screen == Screen::Home => {
            app.notice = None;
            app.input_mode = InputMode::Editing;
        }
        KeyCode::Delete | KeyCode::Backspace if app.screen == Screen::Home => {
            if let Some(selected) = app.feed_list.state.selected() {
                let title = &app.feed_list.items[selected].title;
                app.confirmation_popup = Some(ConfirmationPopup {
                    message: format!("Delete “{title}”?"),
                    choice: ConfirmationChoice::Cancel,
                    kind: ConfirmationKind::DeleteFeed,
                });
            }
        }
        KeyCode::Char('t' | 'T') => request_tts(app, narration)?,
        KeyCode::Esc | KeyCode::Left => go_back(app, narration),
        KeyCode::Down | KeyCode::Char('j') => move_selection(app, Direction::Forward),
        KeyCode::Up | KeyCode::Char('k') => move_selection(app, Direction::Backward),
        KeyCode::PageDown if app.screen == Screen::Article => {
            app.scroll_offset = app.scroll_offset.saturating_add(10).min(app.max_scroll);
        }
        KeyCode::PageUp if app.screen == Screen::Article => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::Char(' ') if app.screen == Screen::Article => {
            toggle_narration(app, narration)?;
        }
        KeyCode::Char('s' | 'S') if app.screen == Screen::Article => {
            stop_narration(app, narration);
        }
        KeyCode::Home => select_edge(app, Edge::First),
        KeyCode::End => select_edge(app, Edge::Last),
        KeyCode::Enter | KeyCode::Right => open_selection(app, narration).await?,
        _ => {}
    }
    Ok(())
}
