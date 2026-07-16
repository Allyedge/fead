use std::sync::Arc;

use crate::app::{App, AppResult, ConfirmationChoice, ConfirmationKind, ConfirmationPopup};
use crate::tts::{
    build_narration_units, download_model, model_dir, model_ready, NarrationEvent, NarrationHandle,
    NarrationTextError, NarrationUiState, TtsModelEvent, TTS,
};
use tokio::sync::mpsc;

pub fn handle_tts_model_event(app: &mut App, event: TtsModelEvent, narration: &NarrationHandle) {
    match event {
        TtsModelEvent::Progress { percent } => {
            app.tts_downloading = true;
            app.show_info(format!("Downloading Kokoro EN model… {percent}%"));
        }
        TtsModelEvent::Finished(result) => {
            app.tts_downloading = false;
            match result {
                Ok(()) => match TTS::load() {
                    Ok(tts) => {
                        let tts = Arc::new(tts);
                        narration.set_engine(Some(Arc::clone(&tts)));
                        app.tts = Some(tts);
                        app.show_info("TTS model ready.");
                    }
                    Err(error) => app.show_error(error),
                },
                Err(error) => app.show_error(format!("TTS download failed: {error}")),
            }
        }
    }
}

pub fn handle_narration_event(app: &mut App, event: NarrationEvent) {
    match event {
        NarrationEvent::State(state) => {
            app.narration = state;
            if matches!(
                state,
                NarrationUiState::Preparing { .. }
                    | NarrationUiState::Playing { .. }
                    | NarrationUiState::Paused { .. }
                    | NarrationUiState::Buffering { .. }
                    | NarrationUiState::Completed
            ) {
                app.notice = None;
            }
        }
        NarrationEvent::Error(message) => {
            app.narration = NarrationUiState::Error;
            app.show_error(message);
        }
    }
}

pub(super) fn request_tts(app: &mut App, narration: &NarrationHandle) -> AppResult<()> {
    if app.tts_downloading {
        app.show_info("Download already in progress.");
        return Ok(());
    }

    if app.tts.is_some() {
        app.show_info("TTS is ready.");
        return Ok(());
    }

    if model_ready() {
        match TTS::load() {
            Ok(tts) => {
                let tts = Arc::new(tts);
                narration.set_engine(Some(Arc::clone(&tts)));
                app.tts = Some(tts);
                app.show_info("TTS model loaded.");
            }
            Err(error) => app.show_error(error),
        }
        return Ok(());
    }

    app.confirmation_popup = Some(ConfirmationPopup {
        message: format!("Download Kokoro EN (~330MB) to {}?", model_dir().display()),
        choice: ConfirmationChoice::Cancel,
        kind: ConfirmationKind::DownloadTtsModel,
    });
    Ok(())
}

pub(super) fn toggle_narration(app: &mut App, narration: &NarrationHandle) -> AppResult<()> {
    match app.narration {
        NarrationUiState::Playing { .. }
        | NarrationUiState::Paused { .. }
        | NarrationUiState::Buffering { .. }
        | NarrationUiState::Preparing { .. } => {
            narration.toggle_pause();
            return Ok(());
        }
        NarrationUiState::Idle | NarrationUiState::Completed | NarrationUiState::Error => {}
    }

    if app.tts.is_none() {
        if model_ready() {
            match TTS::load() {
                Ok(tts) => {
                    let tts = Arc::new(tts);
                    narration.set_engine(Some(Arc::clone(&tts)));
                    app.tts = Some(tts);
                }
                Err(error) => {
                    app.show_error(error);
                    return Ok(());
                }
            }
        } else {
            app.show_info("TTS is not ready. Press t to set it up.");
            return Ok(());
        }
    }

    let Some(content) = app.current_entry.body() else {
        app.show_info("No article content to read.");
        return Ok(());
    };

    let units = match build_narration_units(&app.current_entry.title, content) {
        Ok(units) => units,
        Err(NarrationTextError::NoNarratableContent) => {
            app.show_info("No article content to read.");
            return Ok(());
        }
    };

    app.notice = None;
    app.narration = NarrationUiState::Preparing {
        current: 0,
        total: units.len(),
    };
    narration.play(units);
    Ok(())
}

pub(super) fn stop_narration(app: &mut App, narration: &NarrationHandle) {
    if app.narration.is_active() || matches!(app.narration, NarrationUiState::Completed) {
        narration.stop();
        app.narration = NarrationUiState::Idle;
        app.notice = None;
    }
}

pub(super) fn start_tts_download(
    app: &mut App,
    model_tx: &mpsc::UnboundedSender<TtsModelEvent>,
) -> AppResult<()> {
    if app.tts_downloading {
        app.show_info("TTS model download already in progress.");
        return Ok(());
    }

    app.tts_downloading = true;
    app.show_info("Downloading Kokoro EN model… 0%");

    let tx = model_tx.clone();
    tokio::spawn(async move {
        let result = download_model(|percent| {
            let _ = tx.send(TtsModelEvent::Progress { percent });
        })
        .await;
        let _ = tx.send(TtsModelEvent::Finished(result));
    });

    Ok(())
}
