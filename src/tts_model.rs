use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;

const URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-en-v0_19.tar.bz2";

const MODEL_DIR: &str = "models/kokoro-en-v0_19";

#[derive(Clone, Debug)]
pub enum TtsModelEvent {
    Progress { percent: u8 },
    Finished(Result<(), String>),
}

pub fn model_dir() -> PathBuf {
    PathBuf::from(MODEL_DIR)
}

pub fn model_ready() -> bool {
    let dir = model_dir();
    dir.join("model.onnx").is_file()
        && dir.join("voices.bin").is_file()
        && dir.join("tokens.txt").is_file()
        && dir.join("espeak-ng-data").is_dir()
}

pub async fn download_model(mut on_progress: impl FnMut(u8) + Send) -> Result<(), String> {
    if model_ready() {
        on_progress(100);
        return Ok(());
    }

    let dest = model_dir();
    let models = dest.parent().unwrap_or(Path::new("models"));
    fs::create_dir_all(models).map_err(|e| e.to_string())?;

    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }

    let archive = models.join("kokoro-en-v0_19.tar.bz2.part");
    download_archive(&archive, &mut on_progress).await?;
    on_progress(99);

    let archive_path = archive.clone();
    let models_path = models.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let file = fs::File::open(&archive_path).map_err(|e| e.to_string())?;
        let mut archive = tar::Archive::new(bzip2::read::BzDecoder::new(file));
        archive
            .unpack(&models_path)
            .map_err(|e| format!("extract failed: {e}"))
    })
    .await
    .map_err(|e| e.to_string())??;

    let _ = fs::remove_file(&archive);

    if !model_ready() {
        return Err("model files missing after extract".into());
    }

    on_progress(100);
    Ok(())
}

async fn download_archive(
    path: &Path,
    on_progress: &mut (impl FnMut(u8) + Send),
) -> Result<(), String> {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(60 * 30))
        .user_agent(concat!("fead/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(URL).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("download failed: HTTP {}", response.status()));
    }

    let total = response.content_length();
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut last_percent: u8 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded = downloaded.saturating_add(chunk.len() as u64);

        if let Some(total) = total {
            if total > 0 {
                let percent = ((downloaded.saturating_mul(98)) / total).min(98) as u8;
                if percent != last_percent {
                    last_percent = percent;
                    on_progress(percent);
                }
            }
        }
    }

    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}
