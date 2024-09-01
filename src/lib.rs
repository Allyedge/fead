/// Application.
pub mod app;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Event handler.
pub mod handler;

pub mod feeds;
pub mod fetch;
pub mod reader;

pub trait FormatText {
    fn strip_trailing_newline(&self) -> Self;
}

impl FormatText for String {
    fn strip_trailing_newline(&self) -> String {
        self.strip_suffix("\r\n")
            .or(self.strip_suffix('\n'))
            .unwrap_or(self)
            .to_string()
    }
}
