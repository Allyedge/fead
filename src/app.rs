use std::error;

use crate::{
    entries::Entry,
    feeds::{load_feeds, Feed},
    screen::Screen,
};
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

#[derive(Debug)]
pub struct EntryList {
    pub items: Vec<Entry>,
    pub state: ListState,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub screen: Screen,
    pub input: Input,
    pub input_mode: InputMode,
    pub feed_list: FeedList,
    pub entry_list: EntryList,
    pub current_entry: Entry,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            screen: Screen::Home,
            input: Input::default(),
            input_mode: InputMode::Normal,
            feed_list: FeedList {
                items: load_feeds().unwrap(),
                state: ListState::default(),
            },
            entry_list: EntryList {
                items: vec![],
                state: ListState::default(),
            },
            current_entry: Entry {
                title: String::new(),
                description: String::new(),
                content: String::new(),
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
