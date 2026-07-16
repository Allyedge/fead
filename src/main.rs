use std::io;

use fead::app::{App, AppResult};
use fead::event::{Event, EventHandler};
use fead::handler::{handle_key_events, handle_tts_model_event};
use fead::tts::TtsModelEvent;
use fead::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut app = App::new()?;

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let (model_tx, mut model_rx) = mpsc::unbounded_channel::<TtsModelEvent>();

    let run_result = async {
        while app.running {
            tui.draw(&mut app)?;
            tokio::select! {
                event = model_rx.recv() => {
                    if let Some(event) = event {
                        handle_tts_model_event(&mut app, event);
                    }
                }
                event = tui.events.next() => {
                    match event? {
                        Event::Mouse(_) | Event::Resize(_, _) => {}
                        Event::Key(key_event) => {
                            handle_key_events(key_event, &mut app, &model_tx).await?;
                        }
                    }
                }
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;

    let exit_result = tui.exit();
    run_result?;
    exit_result?;

    Ok(())
}
