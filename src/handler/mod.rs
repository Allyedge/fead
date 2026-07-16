mod confirmation;
mod editing;
mod feed_actions;
mod navigation;
mod normal;
mod tts;

pub use tts::handle_tts_model_event;

use crate::app::{App, AppResult, InputMode};
use crate::tts::TtsModelEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use confirmation::handle_confirmation;
use editing::handle_editing_mode;
use normal::handle_normal_mode;

pub async fn handle_key_events(
    key: KeyEvent,
    app: &mut App,
    model_tx: &mpsc::UnboundedSender<TtsModelEvent>,
) -> AppResult<()> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c' | 'C'))
    {
        app.quit();
        return Ok(());
    }

    if app.confirmation_popup.is_some() {
        handle_confirmation(key, app, model_tx)?;
        return Ok(());
    }

    match app.input_mode {
        InputMode::Normal => handle_normal_mode(key, app).await?,
        InputMode::Editing => handle_editing_mode(key, app).await?,
    }

    Ok(())
}
