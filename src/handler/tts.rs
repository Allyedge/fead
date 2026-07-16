use crate::app::{
    App, AppResult, ConfirmationChoice, ConfirmationKind, ConfirmationPopup,
};
use crate::tts::{download_model, model_dir, model_ready, TtsModelEvent, TTS};
use tokio::sync::mpsc;

pub fn handle_tts_model_event(app: &mut App, event: TtsModelEvent) {
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

pub(super) fn request_tts(app: &mut App) -> AppResult<()> {
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
                app.tts = Some(tts);
                app.show_info("TTS model loaded.");
            }
            Err(error) => app.show_error(error),
        }
        return Ok(());
    }

    app.confirmation_popup = Some(ConfirmationPopup {
        message: format!(
            "Download Kokoro EN (~330MB) to {}?",
            model_dir().display()
        ),
        choice: ConfirmationChoice::Cancel,
        kind: ConfirmationKind::DownloadTtsModel,
    });
    Ok(())
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
