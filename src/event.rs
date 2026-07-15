use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::app::AppResult;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();

            while let Some(Ok(event)) = reader.next().await {
                let event = match event {
                    CrosstermEvent::Key(key)
                        if key.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        Some(Event::Key(key))
                    }
                    CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
                    CrosstermEvent::Resize(width, height) => Some(Event::Resize(width, height)),
                    _ => None,
                };

                if event.is_some_and(|event| sender.send(event).is_err()) {
                    break;
                }
            }
        });

        Self { receiver, handler }
    }

    pub async fn next(&mut self) -> AppResult<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| Box::new(std::io::Error::other("terminal event stream closed")).into())
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.handler.abort();
    }
}
