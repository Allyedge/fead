use std::io;

use fead::app::{App, AppResult};
use fead::event::{Event, EventHandler};
use fead::handler::handle_key_events;
use fead::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut app = App::new()?;

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let run_result = async {
        while app.running {
            tui.draw(&mut app)?;
            match tui.events.next().await? {
                Event::Mouse(_) | Event::Resize(_, _) => {}
                Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
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
