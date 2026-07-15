use std::error;

use crate::{
    entries::Entry,
    feeds::{load_feeds, Feed},
    screen::Screen,
};
use ratatui::widgets::ListState;
use tui_input::Input;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Eq, PartialEq)]
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

#[derive(Debug)]
pub struct ConfirmationPopup {
    pub message: String,
    pub choice: ConfirmationChoice,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationChoice {
    Cancel,
    Delete,
}

impl ConfirmationChoice {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Cancel => Self::Delete,
            Self::Delete => Self::Cancel,
        };
    }
}

#[derive(Debug)]
pub enum Notice {
    Error(String),
    Info(String),
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub input: Input,
    pub input_mode: InputMode,
    pub feed_list: FeedList,
    pub entry_list: EntryList,
    pub current_entry: Entry,
    pub scroll_offset: u16,
    pub max_scroll: u16,
    pub confirmation_popup: Option<ConfirmationPopup>,
    pub notice: Option<Notice>,
}

impl App {
    pub fn new() -> AppResult<Self> {
        let feeds = load_feeds()?;
        let mut feed_state = ListState::default();
        if !feeds.is_empty() {
            feed_state.select_first();
        }

        Ok(Self {
            running: true,
            screen: Screen::Home,
            input: Input::default(),
            input_mode: InputMode::Normal,
            feed_list: FeedList {
                items: feeds,
                state: feed_state,
            },
            entry_list: EntryList {
                items: vec![],
                state: ListState::default(),
            },
            current_entry: Entry::default(),
            scroll_offset: 0,
            max_scroll: 0,
            confirmation_popup: None,
            notice: None,
        })
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn show_error(&mut self, message: impl Into<String>) {
        self.notice = Some(Notice::Error(message.into()));
    }

    pub fn show_info(&mut self, message: impl Into<String>) {
        self.notice = Some(Notice::Info(message.into()));
    }

    pub fn update_article_viewport(&mut self, line_count: usize, viewport_height: u16) {
        self.max_scroll = line_count.saturating_sub(viewport_height as usize) as u16;
        self.scroll_offset = self.scroll_offset.min(self.max_scroll);
    }
}
