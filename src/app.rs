use std::error;

use crate::feeds::{load, Feed};
use ratatui::widgets::ListState;
use tui_input::Input;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug)]
pub struct FeedList {
    pub items: Vec<Feed>,
    pub state: ListState,
}
/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub input: Input,
    pub input_mode: InputMode,
    pub feed_list: FeedList,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            input: Input::default(),
            input_mode: InputMode::Normal,
            feed_list: FeedList {
                items: load().unwrap(),
                state: ListState::default(),
            },
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
