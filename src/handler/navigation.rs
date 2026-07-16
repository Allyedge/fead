use crate::app::App;
use crate::screen::Screen;
use crate::tts::NarrationHandle;

use super::tts::stop_narration;

#[derive(Clone, Copy)]
pub(super) enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Copy)]
pub(super) enum Edge {
    First,
    Last,
}

pub(super) fn go_back(app: &mut App, narration: &NarrationHandle) {
    app.notice = None;
    match app.screen {
        Screen::Home => {}
        Screen::Feed => app.screen = Screen::Home,
        Screen::Article => {
            stop_narration(app, narration);
            app.scroll_offset = 0;
            app.screen = Screen::Feed;
        }
    }
}

pub(super) fn move_selection(app: &mut App, direction: Direction) {
    match (app.screen, direction) {
        (Screen::Home, Direction::Forward) => app.feed_list.state.select_next(),
        (Screen::Home, Direction::Backward) => app.feed_list.state.select_previous(),
        (Screen::Feed, Direction::Forward) => app.entry_list.state.select_next(),
        (Screen::Feed, Direction::Backward) => app.entry_list.state.select_previous(),
        (Screen::Article, Direction::Forward) => {
            app.scroll_offset = app.scroll_offset.saturating_add(1).min(app.max_scroll);
        }
        (Screen::Article, Direction::Backward) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }
    }
}

pub(super) fn select_edge(app: &mut App, edge: Edge) {
    match (app.screen, edge) {
        (Screen::Home, Edge::First) => app.feed_list.state.select_first(),
        (Screen::Home, Edge::Last) => app.feed_list.state.select_last(),
        (Screen::Feed, Edge::First) => app.entry_list.state.select_first(),
        (Screen::Feed, Edge::Last) => app.entry_list.state.select_last(),
        (Screen::Article, Edge::First) => app.scroll_offset = 0,
        (Screen::Article, Edge::Last) => app.scroll_offset = app.max_scroll,
    }
}
