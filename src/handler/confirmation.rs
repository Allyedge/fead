use crate::app::{App, AppResult, ConfirmationChoice, ConfirmationKind};
use crate::tts::TtsModelEvent;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

use super::feed_actions::delete_selected_feed;
use super::tts::start_tts_download;

pub(super) fn handle_confirmation(
    key: KeyEvent,
    app: &mut App,
    model_tx: &mpsc::UnboundedSender<TtsModelEvent>,
) -> AppResult<()> {
    match key.code {
        KeyCode::Esc => app.confirmation_popup = None,
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab => {
            if let Some(popup) = &mut app.confirmation_popup {
                popup.choice.toggle();
            }
        }
        KeyCode::Enter => {
            let Some(popup) = app.confirmation_popup.take() else {
                return Ok(());
            };
            if popup.choice != ConfirmationChoice::Accept {
                return Ok(());
            }
            match popup.kind {
                ConfirmationKind::DeleteFeed => delete_selected_feed(app)?,
                ConfirmationKind::DownloadTtsModel => start_tts_download(app, model_tx)?,
            }
        }
        _ => {}
    }
    Ok(())
}
