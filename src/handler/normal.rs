use crate::app::{
    App, AppResult, ConfirmationChoice, ConfirmationKind, ConfirmationPopup, InputMode,
};
use crate::screen::Screen;
use crossterm::event::{KeyCode, KeyEvent};

use super::feed_actions::open_selection;
use super::navigation::{go_back, move_selection, select_edge, Direction, Edge};
use super::tts::request_tts;

pub(super) async fn handle_normal_mode(key: KeyEvent, app: &mut App) -> AppResult<()> {
    match key.code {
        KeyCode::Char('q') => app.quit(),
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
        KeyCode::Char('t' | 'T') => request_tts(app)?,
        KeyCode::Esc | KeyCode::Left => go_back(app),
        KeyCode::Down | KeyCode::Char('j') => move_selection(app, Direction::Forward),
        KeyCode::Up | KeyCode::Char('k') => move_selection(app, Direction::Backward),
        KeyCode::PageDown if app.screen == Screen::Article => {
            app.scroll_offset = app.scroll_offset.saturating_add(10).min(app.max_scroll);
        }
        KeyCode::PageUp if app.screen == Screen::Article => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::Home => select_edge(app, Edge::First),
        KeyCode::End => select_edge(app, Edge::Last),
        KeyCode::Enter | KeyCode::Right => open_selection(app).await?,
        _ => {}
    }
    Ok(())
}
